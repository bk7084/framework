use crate::core::assets::Asset;
use crossbeam_channel::{Receiver, Sender};
use std::{
    cmp::Ordering,
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    sync::atomic::AtomicU32,
};

/// Handle to an asset.
pub struct Handle<T: Asset> {
    pub(crate) generation: u32,
    pub(crate) index: u32,
    _marker: PhantomData<T>,
}

impl<T: Asset> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T: Asset> Eq for Handle<T> {}

impl<T: Asset> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // First compare generations then indices.
        Some(
            self.generation
                .cmp(&other.generation)
                .then(self.index.cmp(&other.index)),
        )
    }
}

impl<T: Asset> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare generations then indices.
        self.generation
            .cmp(&other.generation)
            .then(self.index.cmp(&other.index))
    }
}

impl<T: Asset> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

impl<T: Asset> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            generation: self.generation,
            index: self.index,
            _marker: Default::default(),
        }
    }
}

impl<T: Asset> Copy for Handle<T> {}

impl<T: Asset> Debug for Handle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("idx", &self.index)
            .field("gen", &self.generation)
            .finish()
    }
}

/// Runtime handle allocator.
pub(crate) struct HandleAllocator<T>
where
    T: Asset,
{
    /// Monotonically increasing index.
    pub(crate) next_index: AtomicU32,
    recycle_queue_sender: Sender<Handle<T>>,
    /// Freelist of recycled handles. It serves as a queue of recycled handles.
    recycle_queue_receiver: Receiver<Handle<T>>,
    recycle_sender: Sender<Handle<T>>,
    /// Freelist of recycled handles, which is used to unload assets.
    /// See [`Assets::flush`].
    pub(crate) recycle_receiver: Receiver<Handle<T>>,
}

impl<T> HandleAllocator<T>
where
    T: Asset,
{
    /// Create a new handle allocator.
    pub fn new() -> Self {
        let (recycle_queue_sender, recycle_queue_receiver) = crossbeam_channel::unbounded();
        let (recycle_sender, recycle_receiver) = crossbeam_channel::unbounded();
        Self {
            next_index: AtomicU32::new(0),
            recycle_queue_sender,
            recycle_queue_receiver,
            recycle_sender,
            recycle_receiver,
        }
    }

    /// Reserve a new handle.
    ///
    /// If there are any recycled handles, one of those will be returned.
    /// Otherwise, a new handle will be created. When a handle is recycled,
    /// it is sent to the recycle queue.
    ///
    /// When a new handle is reserved, it is first checked if there are any
    /// recycled handles. If there are, one of those is returned. Otherwise,
    /// a new handle is created.
    pub fn reserve(&self) -> Handle<T> {
        if let Ok(mut recycled) = self.recycle_queue_receiver.try_recv() {
            recycled.generation += 1;
            self.recycle_sender.send(recycled).unwrap();
            return recycled;
        }
        Handle {
            generation: 0,
            index: self
                .next_index
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            _marker: Default::default(),
        }
    }

    /// Recycle a handle.
    pub fn recycle(&self, handle: Handle<T>) {
        self.recycle_queue_sender.send(handle).unwrap();
    }
}

impl<T: Asset> Default for HandleAllocator<T> {
    fn default() -> Self {
        Self::new()
    }
}
