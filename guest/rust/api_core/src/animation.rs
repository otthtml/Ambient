use std::{cell::RefCell, rc::Rc};

use crate::{
    components::core::{
        animation::{
            animation_graph, blend, clip_duration, freeze_at_percentage, freeze_at_time,
            mask_bind_ids, mask_weights, ref_count, start_time,
        },
        app::name,
        ecs::{children, parent},
    },
    entity::{
        add_component, despawn_recursive, get_component, mutate_component, remove_component,
        set_component,
    },
    prelude::{block_until, time, Entity, EntityId},
};

/// tmp
#[derive(Debug, Clone, Copy)]
pub struct AnimationGraph(pub EntityId);
impl AnimationGraph {
    /// tmp
    pub fn new(root: impl AsRef<AnimationNode>) -> Self {
        let root: &AnimationNode = root.as_ref();
        let graph = Entity::new()
            .with_default(animation_graph())
            .with(children(), vec![root.0])
            .with(name(), "Animation graph".to_string())
            .spawn();
        add_component(root.0, parent(), graph);
        Self(graph)
    }
    /// tmp
    fn root(&self) -> Option<EntityId> {
        if let Some(children) = get_component(self.0, children()) {
            children.get(0).map(|x| *x)
        } else {
            None
        }
    }
    /// tmp
    pub fn set_root(&self, new_root: impl AsRef<AnimationNode>) {
        if let Some(root) = self.root() {
            remove_component(root, parent());
        }
        let new_root: &AnimationNode = new_root.as_ref();
        add_component(self.0, children(), vec![new_root.0]);
        add_component(new_root.0, parent(), self.0);
    }
}
/// tmp
#[derive(Debug)]
pub struct AnimationNode(pub EntityId);
impl Clone for AnimationNode {
    fn clone(&self) -> Self {
        mutate_component(self.0, ref_count(), |x| *x += 1);
        Self(self.0.clone())
    }
}
impl Drop for AnimationNode {
    fn drop(&mut self) {
        mutate_component(self.0, ref_count(), |x| *x -= 1);
    }
}

/// tmp
#[derive(Debug)]
pub struct PlayClipFromUrlNode(pub AnimationNode);
impl PlayClipFromUrlNode {
    /// tmp
    pub fn new(url: impl Into<String>, looping: bool) -> Self {
        use crate::components::core::animation;
        let node = Entity::new()
            .with(animation::play_clip_from_url(), url.into())
            .with(name(), "Play clip from url".to_string())
            .with(animation::looping(), looping)
            .with(start_time(), time())
            .with(ref_count(), 1)
            .spawn();
        Self(AnimationNode(node))
    }
    /// Freeze the animation at time
    pub fn freeze_at_time(&self, time: f32) {
        add_component(self.0 .0, freeze_at_time(), time);
    }
    /// Freeze the animation at time = percentage * duration
    pub fn freeze_at_percentage(&self, percentage: f32) {
        add_component(self.0 .0, freeze_at_percentage(), percentage);
    }
    /// Returns None if the duration hasn't been loaded yet
    pub fn peek_clip_duration(&self) -> Option<f32> {
        get_component(self.0 .0, clip_duration())
    }
    /// Returns the duration of this clip. This is async because it needs to wait for the clip to load before the duration can be returned.
    pub async fn clip_duration(&self) -> f32 {
        let res = Rc::new(RefCell::new(0.));
        {
            let res = res.clone();
            block_until(move || match self.peek_clip_duration() {
                Some(val) => {
                    *res.borrow_mut() = val;
                    true
                }
                None => false,
            })
            .await;
        }
        let val: f32 = *res.borrow();
        val
    }
}
impl AsRef<AnimationNode> for PlayClipFromUrlNode {
    fn as_ref(&self) -> &AnimationNode {
        &self.0
    }
}

/// tmp
#[derive(Debug, Clone)]
pub struct BlendNode(pub AnimationNode);
impl BlendNode {
    /// tmp
    pub fn new(
        left: impl AsRef<AnimationNode>,
        right: impl AsRef<AnimationNode>,
        weight: f32,
    ) -> Self {
        use crate::components::core::animation;
        let left: &AnimationNode = left.as_ref();
        let right: &AnimationNode = right.as_ref();
        let node = Entity::new()
            .with(animation::blend(), weight)
            .with(name(), "Blend".to_string())
            .with(children(), vec![left.0, right.0])
            .with(ref_count(), 1)
            .spawn();
        add_component(left.0, parent(), node);
        add_component(right.0, parent(), node);
        Self(AnimationNode(node))
    }
    /// tmp
    pub fn set_weight(&self, weight: f32) {
        set_component(self.0 .0, blend(), weight);
    }
    /// Sets the mask to a list of (bind_id, weights)
    pub fn set_mask(&self, weights: Vec<(String, f32)>) {
        let (bind_ids, weights): (Vec<_>, Vec<_>) = weights.into_iter().unzip();
        add_component(self.0 .0, mask_bind_ids(), bind_ids);
        add_component(self.0 .0, mask_weights(), weights);
    }
}
impl AsRef<AnimationNode> for BlendNode {
    fn as_ref(&self) -> &AnimationNode {
        &self.0
    }
}
