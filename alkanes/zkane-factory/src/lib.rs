//! ZKane Factory Contract
//!
//! Factory contract for spawning ZKane privacy pool instances.
//! Uses the cellpack pattern where [4, n] deploys the zkane WASM and [6, n] spawns instances.

use alkanes_runtime::{declare_alkane, message::MessageDispatch, runtime::AlkaneResponder};
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::response::CallResponse;
use alkanes_support::context::Context;
use alkanes_support::parcel::AlkaneTransfer;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use metashrew_support::index_pointer::KeyValuePointer;
use zkane_common::ZKaneConfig;
use anyhow::{anyhow, Result};
use std::sync::Arc;

#[cfg(test)]
pub mod tests;

/// ZKane factory contract constants
pub const ZKANE_TEMPLATE_BLOCK: u128 = 4; // Block where zkane WASM is deployed
pub const ZKANE_INSTANCE_BLOCK: u128 = 6; // Block for zkane instances

/// ZKane factory contract
#[derive(Default)]
pub struct ZKaneFactory {
    /// Whether the factory has been initialized
    initialized: bool,
}

/// Message enum for opcode-based dispatch
#[derive(MessageDispatch)]
enum ZKaneFactoryMessage {
    /// Initialize the factory
    #[opcode(0)]
    Initialize,

    /// Deploy or get a zkane pool for an asset
    /// Uses witness envelope for large configuration data
    #[opcode(1)]
    GetOrCreatePool {
        /// Asset ID block
        asset_id_block: u128,
        /// Asset ID tx
        asset_id_tx: u128,
        /// Denomination for the pool
        denomination: u128,
    },

    /// Get the zkane instance ID for an asset/denomination pair
    #[opcode(2)]
    #[returns(Vec<u8>)]
    GetPoolId {
        /// Asset ID block
        asset_id_block: u128,
        /// Asset ID tx
        asset_id_tx: u128,
        /// Denomination for the pool
        denomination: u128,
    },

    /// Check if a pool exists for an asset/denomination pair
    #[opcode(3)]
    #[returns(bool)]
    PoolExists {
        /// Asset ID block
        asset_id_block: u128,
        /// Asset ID tx
        asset_id_tx: u128,
        /// Denomination for the pool
        denomination: u128,
    },

    /// Get all pools for an asset
    #[opcode(4)]
    #[returns(Vec<u8>)]
    GetAssetPools {
        /// Asset ID block
        asset_id_block: u128,
        /// Asset ID tx
        asset_id_tx: u128,
    },

    /// Get factory statistics
    #[opcode(5)]
    #[returns(Vec<u8>)]
    GetStats,
}

impl ZKaneFactory {
    /// Get the pointer to the pool registry
    fn pools_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/pools")
    }

    /// Get the pointer for a specific asset/denomination pool
    fn pool_pointer(&self, asset_id: &AlkaneId, denomination: u128) -> StoragePointer {
        let mut key = Vec::new();
        key.extend_from_slice(&asset_id.block.to_le_bytes());
        key.extend_from_slice(&asset_id.tx.to_le_bytes());
        key.extend_from_slice(&denomination.to_le_bytes());
        
        self.pools_pointer().select(&key)
    }

    /// Get the pointer to the pool count
    fn pool_count_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/pool_count")
    }

    /// Get the number of pools created
    fn get_pool_count(&self) -> u128 {
        self.pool_count_pointer().get_value::<u128>()
    }

    /// Increment the pool count
    fn increment_pool_count(&self) {
        let count = self.get_pool_count();
        self.pool_count_pointer().set_value::<u128>(count + 1);
    }

    /// Get the pointer to asset pools list
    fn asset_pools_pointer(&self, asset_id: &AlkaneId) -> StoragePointer {
        let mut key = Vec::new();
        key.extend_from_slice(&asset_id.block.to_le_bytes());
        key.extend_from_slice(&asset_id.tx.to_le_bytes());
        
        StoragePointer::from_keyword("/asset_pools").select(&key)
    }

    /// Add a pool to the asset pools list
    fn add_to_asset_pools(&self, asset_id: &AlkaneId, denomination: u128, pool_id: &AlkaneId) {
        let asset_pools_ptr = self.asset_pools_pointer(asset_id);
        
        // Get current count for this asset
        let count_ptr = asset_pools_ptr.select(&b"count".to_vec());
        let count = count_ptr.get_value::<u128>();
        
        // Store the new pool info
        let pool_info = serde_json::json!({
            "denomination": denomination,
            "pool_id": {
                "block": pool_id.block,
                "tx": pool_id.tx
            }
        });
        
        let pool_ptr = asset_pools_ptr.select(&count.to_le_bytes().to_vec());
        pool_ptr.set(Arc::new(pool_info.to_string().into_bytes()));
        
        // Update count
        count_ptr.set_value::<u128>(count + 1);
    }

    /// Check if a pool exists for the given asset and denomination
    fn pool_exists(&self, asset_id: &AlkaneId, denomination: u128) -> bool {
        let pool_ptr = self.pool_pointer(asset_id, denomination);
        !pool_ptr.get().is_empty()
    }

    /// Get the pool ID for the given asset and denomination
    fn get_pool_id(&self, asset_id: &AlkaneId, denomination: u128) -> Option<AlkaneId> {
        let pool_ptr = self.pool_pointer(asset_id, denomination);
        let data = pool_ptr.get();
        
        if data.is_empty() {
            return None;
        }
        
        // Deserialize the AlkaneId
        if data.len() >= 32 {
            let block = u128::from_le_bytes(data[0..16].try_into().ok()?);
            let tx = u128::from_le_bytes(data[16..32].try_into().ok()?);
            Some(AlkaneId { block, tx })
        } else {
            None
        }
    }

    /// Store a pool ID for the given asset and denomination
    fn store_pool_id(&self, asset_id: &AlkaneId, denomination: u128, pool_id: &AlkaneId) {
        let pool_ptr = self.pool_pointer(asset_id, denomination);
        
        let mut data = Vec::new();
        data.extend_from_slice(&pool_id.block.to_le_bytes());
        data.extend_from_slice(&pool_id.tx.to_le_bytes());
        
        pool_ptr.set(Arc::new(data));
        
        // Add to asset pools list
        self.add_to_asset_pools(asset_id, denomination, pool_id);
    }

    /// Generate a unique pool ID based on asset and denomination
    fn generate_pool_id(&self, asset_id: &AlkaneId, denomination: u128) -> AlkaneId {
        // Use a hash of asset_id and denomination to generate a unique tx value
        let mut hasher_input = Vec::new();
        hasher_input.extend_from_slice(&asset_id.block.to_le_bytes());
        hasher_input.extend_from_slice(&asset_id.tx.to_le_bytes());
        hasher_input.extend_from_slice(&denomination.to_le_bytes());
        
        // Simple hash for demo - in production use proper hash function
        let mut hash_value = 0u128;
        for chunk in hasher_input.chunks(16) {
            let mut bytes = [0u8; 16];
            bytes[..chunk.len()].copy_from_slice(chunk);
            hash_value ^= u128::from_le_bytes(bytes);
        }
        
        AlkaneId {
            block: ZKANE_INSTANCE_BLOCK,
            tx: hash_value,
        }
    }

    /// Observe initialization to prevent multiple initializations
    fn observe_initialization(&self) -> Result<()> {
        let mut pointer = StoragePointer::from_keyword("/initialized");
        if pointer.get().is_empty() {
            pointer.set_value::<u8>(1);
            Ok(())
        } else {
            Err(anyhow!("Factory already initialized"))
        }
    }

    /// Initialize the factory
    fn initialize(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);

        // Prevent multiple initializations
        self.observe_initialization()?;

        // Initialize pool count
        self.pool_count_pointer().set_value::<u128>(0);

        Ok(response)
    }

    /// Get or create a zkane pool for the given asset and denomination
    fn get_or_create_pool(
        &self,
        asset_id_block: u128,
        asset_id_tx: u128,
        denomination: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let asset_id = AlkaneId {
            block: asset_id_block,
            tx: asset_id_tx,
        };

        // Check if pool already exists
        if let Some(existing_pool_id) = self.get_pool_id(&asset_id, denomination) {
            // Pool exists, forward the incoming alkanes to it
            let pool_cellpack = Cellpack {
                target: existing_pool_id,
                inputs: vec![1], // Deposit opcode
            };

            // Forward all incoming alkanes to the existing pool
            let pool_response = self.call(
                &pool_cellpack,
                &context.incoming_alkanes,
                <Self as AlkaneResponder>::fuel(&self),
            )?;

            // Return the pool's response
            return Ok(pool_response);
        }

        // Pool doesn't exist, create it
        let pool_id = self.generate_pool_id(&asset_id, denomination);

        // Read configuration from witness envelope if provided
        let witness_data = alkanes_support::witness::find_witness_payload(
            &context.transaction()?,
            0,
        ).unwrap_or_default();

        let tree_height = if !witness_data.is_empty() {
            // Try to parse tree height from witness data
            if witness_data.len() >= 4 {
                u32::from_le_bytes(witness_data[0..4].try_into().unwrap_or([20, 0, 0, 0]))
            } else {
                20 // Default tree height
            }
        } else {
            20 // Default tree height
        };

        // Create the pool using cellpack to [6, pool_id.tx]
        let init_cellpack = Cellpack {
            target: pool_id.clone(),
            inputs: vec![
                0, // Initialize opcode
                asset_id_block,
                asset_id_tx,
                denomination,
                tree_height as u128,
            ],
        };

        // Call the pool initialization
        let init_response = self.call(
            &init_cellpack,
            &alkanes_support::parcel::AlkaneTransferParcel::default(),
            <Self as AlkaneResponder>::fuel(&self),
        )?;

        // Store the pool ID in our registry
        self.store_pool_id(&asset_id, denomination, &pool_id);
        self.increment_pool_count();

        // Now forward the deposit to the newly created pool
        let deposit_cellpack = Cellpack {
            target: pool_id.clone(),
            inputs: vec![1], // Deposit opcode
        };

        let deposit_response = self.call(
            &deposit_cellpack,
            &context.incoming_alkanes,
            <Self as AlkaneResponder>::fuel(&self),
        )?;

        // Return information about the created pool
        let pool_info = serde_json::json!({
            "created": true,
            "pool_id": {
                "block": pool_id.block,
                "tx": pool_id.tx
            },
            "asset_id": {
                "block": asset_id.block,
                "tx": asset_id.tx
            },
            "denomination": denomination,
            "tree_height": tree_height
        });

        response.data = pool_info.to_string().into_bytes();
        response.alkanes = deposit_response.alkanes;

        Ok(response)
    }

    /// Get the pool ID for an asset/denomination pair
    fn get_pool_id_response(
        &self,
        asset_id_block: u128,
        asset_id_tx: u128,
        denomination: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let asset_id = AlkaneId {
            block: asset_id_block,
            tx: asset_id_tx,
        };

        if let Some(pool_id) = self.get_pool_id(&asset_id, denomination) {
            let mut data = Vec::new();
            data.extend_from_slice(&pool_id.block.to_le_bytes());
            data.extend_from_slice(&pool_id.tx.to_le_bytes());
            response.data = data;
        } else {
            response.data = vec![]; // Empty response if pool doesn't exist
        }

        Ok(response)
    }

    /// Check if a pool exists
    fn pool_exists_response(
        &self,
        asset_id_block: u128,
        asset_id_tx: u128,
        denomination: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let asset_id = AlkaneId {
            block: asset_id_block,
            tx: asset_id_tx,
        };

        let exists = self.pool_exists(&asset_id, denomination);
        response.data = vec![if exists { 1u8 } else { 0u8 }];

        Ok(response)
    }

    /// Get all pools for an asset
    fn get_asset_pools_response(
        &self,
        asset_id_block: u128,
        asset_id_tx: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let asset_id = AlkaneId {
            block: asset_id_block,
            tx: asset_id_tx,
        };

        let asset_pools_ptr = self.asset_pools_pointer(&asset_id);
        let count_ptr = asset_pools_ptr.select(&b"count".to_vec());
        let count = count_ptr.get_value::<u128>();

        let mut pools = Vec::new();
        for i in 0..count {
            let pool_ptr = asset_pools_ptr.select(&i.to_le_bytes().to_vec());
            let pool_data = pool_ptr.get();
            if !pool_data.is_empty() {
                if let Ok(pool_info) = String::from_utf8(pool_data.to_vec()) {
                    pools.push(pool_info);
                }
            }
        }

        let result = serde_json::json!({
            "asset_id": {
                "block": asset_id.block,
                "tx": asset_id.tx
            },
            "pools": pools
        });

        response.data = result.to_string().into_bytes();
        Ok(response)
    }

    /// Get factory statistics
    fn get_stats_response(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);

        let pool_count = self.get_pool_count();

        let stats = serde_json::json!({
            "total_pools": pool_count,
            "factory_version": "1.0.0",
            "zkane_template_block": ZKANE_TEMPLATE_BLOCK,
            "zkane_instance_block": ZKANE_INSTANCE_BLOCK
        });

        response.data = stats.to_string().into_bytes();
        Ok(response)
    }
}

impl AlkaneResponder for ZKaneFactory {}

// Use the MessageDispatch macro for opcode handling
declare_alkane! {
    impl AlkaneResponder for ZKaneFactory {
        type Message = ZKaneFactoryMessage;
    }
}