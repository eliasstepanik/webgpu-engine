//! Physics world resource managing the Rapier simulation
//!
//! This module provides the PhysicsWorld struct that wraps all Rapier
//! structures needed for physics simulation with f64 precision.

use crate::core::entity::Entity;
use rapier3d_f64::prelude::*;
use std::collections::HashMap;
use tracing::{debug, info};

/// Physics world resource containing all Rapier structures
pub struct PhysicsWorld {
    /// Set of rigid bodies in the simulation
    pub rigid_body_set: RigidBodySet,
    
    /// Set of colliders in the simulation
    pub collider_set: ColliderSet,
    
    /// Integration parameters for the simulation
    pub integration_parameters: IntegrationParameters,
    
    /// Physics pipeline for stepping the simulation
    pub physics_pipeline: PhysicsPipeline,
    
    /// Island manager for grouping connected bodies
    pub island_manager: IslandManager,
    
    /// Broad phase for coarse collision detection
    pub broad_phase: BroadPhase,
    
    /// Narrow phase for precise collision detection
    pub narrow_phase: NarrowPhase,
    
    /// Set of impulse-based joints
    pub impulse_joint_set: ImpulseJointSet,
    
    /// Set of multibody (articulated) joints
    pub multibody_joint_set: MultibodyJointSet,
    
    /// CCD solver for continuous collision detection
    pub ccd_solver: CCDSolver,
    
    /// Gravity vector for the simulation
    pub gravity: Vector<f64>,
    
    /// Query pipeline for raycasts and shape queries
    pub query_pipeline: QueryPipeline,
    
    /// Mapping from entity to rigid body handle
    entity_to_body: HashMap<Entity, RigidBodyHandle>,
    
    /// Mapping from rigid body handle to entity
    body_to_entity: HashMap<RigidBodyHandle, Entity>,
    
    /// Mapping from entity to collider handles (an entity can have multiple colliders)
    entity_to_colliders: HashMap<Entity, Vec<ColliderHandle>>,
}

impl PhysicsWorld {
    /// Create a new physics world with default settings
    pub fn new() -> Self {
        info!("Initializing physics world with f64 precision");
        
        let mut integration_parameters = IntegrationParameters::default();
        // Set fixed timestep for deterministic simulation (60 Hz)
        integration_parameters.dt = 1.0 / 60.0;
        
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            integration_parameters,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            gravity: vector![0.0, -9.81, 0.0], // Standard Earth gravity
            query_pipeline: QueryPipeline::new(),
            entity_to_body: HashMap::new(),
            body_to_entity: HashMap::new(),
            entity_to_colliders: HashMap::new(),
        }
    }
    
    /// Set the gravity vector for the simulation
    pub fn set_gravity(&mut self, gravity: Vector<f64>) {
        self.gravity = gravity;
        debug!("Physics gravity set to: {:?}", gravity);
    }
    
    /// Register a rigid body with an entity
    pub fn register_body(&mut self, entity: Entity, handle: RigidBodyHandle) {
        self.entity_to_body.insert(entity, handle);
        self.body_to_entity.insert(handle, entity);
    }
    
    /// Unregister a rigid body from an entity
    pub fn unregister_body(&mut self, entity: Entity) -> Option<RigidBodyHandle> {
        if let Some(handle) = self.entity_to_body.remove(&entity) {
            self.body_to_entity.remove(&handle);
            Some(handle)
        } else {
            None
        }
    }
    
    /// Get the rigid body handle for an entity
    pub fn get_body_handle(&self, entity: Entity) -> Option<RigidBodyHandle> {
        self.entity_to_body.get(&entity).copied()
    }
    
    /// Get the entity for a rigid body handle
    pub fn get_entity_for_body(&self, handle: RigidBodyHandle) -> Option<Entity> {
        self.body_to_entity.get(&handle).copied()
    }
    
    /// Register a collider with an entity
    pub fn register_collider(&mut self, entity: Entity, handle: ColliderHandle) {
        self.entity_to_colliders
            .entry(entity)
            .or_insert_with(Vec::new)
            .push(handle);
    }
    
    /// Unregister all colliders for an entity
    pub fn unregister_colliders(&mut self, entity: Entity) -> Vec<ColliderHandle> {
        self.entity_to_colliders.remove(&entity).unwrap_or_default()
    }
    
    /// Get all collider handles for an entity
    pub fn get_collider_handles(&self, entity: Entity) -> &[ColliderHandle] {
        self.entity_to_colliders
            .get(&entity)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
    
    /// Step the physics simulation
    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        );
        
        // Update query pipeline after physics step
        self.query_pipeline.update(&self.collider_set);
    }
    
    /// Perform a raycast in the physics world
    pub fn raycast(
        &self,
        ray_origin: Point<f64>,
        ray_dir: Vector<f64>,
        max_distance: f64,
        solid: bool,
        filter: QueryFilter,
    ) -> Option<(ColliderHandle, f64)> {
        let ray = Ray::new(ray_origin, ray_dir);
        self.query_pipeline.cast_ray(
            &self.collider_set,
            &ray,
            max_distance,
            solid,
            filter,
        )
    }
    
    /// Clean up resources for a removed entity
    pub fn cleanup_entity(&mut self, entity: Entity) {
        // Remove rigid body if it exists
        if let Some(body_handle) = self.unregister_body(entity) {
            self.rigid_body_set.remove(
                body_handle,
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                true, // Also remove attached colliders
            );
        }
        
        // Remove any remaining colliders
        let collider_handles = self.unregister_colliders(entity);
        for handle in collider_handles {
            if self.collider_set.contains(handle) {
                self.collider_set.remove(
                    handle,
                    &mut self.island_manager,
                    &mut self.rigid_body_set,
                    false, // Don't wake up bodies
                );
            }
        }
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}