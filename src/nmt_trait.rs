// pub trait NMT<M: MerkleTree, H: Hasher> {
//     type Namespace: NamespaceTrait;

//     pub fn highest_ns() -> u64;

//     pub fn increment_highest_ns(&mut self);

//     pub fn push_namespaced_leaf(
//         &mut self,
//         raw_data: &[u8],
//         namespace: Namespace,
//     ) -> Result<(), ()> {
//         self.push_leaf(raw_data);
//     }
// }

use std::{hash::Hash, marker::PhantomData};

// use sha2::{Digest, Sha256};

use crate::{
    simple_merkle::{
        db::PreimageDb,
        tree::{MerkleHash, MerkleTree as SimpleMerkle},
    },
    NamespaceId, NamespaceMerkleHasher, NamespacedSha2Hasher,
};

// pub type Hasher = Sha256;

// type Namespace = u64;
// pub trait NMTHash {
//     type Output: NamespacedDigest;
//     const EMPTY_ROOT: Self::Output;

//     fn hash_leaf(data: &[u8]) -> Self::Output;
//     fn hash_nodes(l: &Self::Output, r: &Self::Output) -> Self::Output {
//         let (left_min_ns, left_max_ns) = l.get_namespace_range();
//         let (right_min_ns, right_max_ns) = r.get_namespace_range();

//         if left_max_ns > right_min_ns {
//             panic!("leaves are out of order")
//         }

//         let mut hasher = Hasher::new();
//         hasher.update(left_min_ns.to_be_bytes());
//         hasher.update(right_max_ns.to_be_bytes());
//         hasher.update(l.as_ref());
//         hasher.update(r.as_ref());

//         let digest = Self::Output::new(left_min_ns, right_max_ns, hasher.finalize().as_ref());
//         digest
//     }
// }

// pub trait Namespaced {
//     fn get_namespace(&self) -> Namespace;
// }

// pub trait NamespacedDigest: Hash + AsRef<[u8]> {
//     fn new(min_ns: Namespace, max_ns: Namespace, hash: &[u8]) -> Self;
//     fn get_namespace_range(&self) -> (Namespace, Namespace);
// }

pub struct NMT<M: MerkleTree<H>, H: NamespaceMerkleHasher> {
    inner: M,
    max_ns: NamespaceId,
    phantom: PhantomData<H>,
}

pub trait MerkleTree<H: MerkleHash> {
    type Output;
    type Leaf: AsRef<[u8]>;

    fn push_leaf(&mut self, _leaf: Self::Leaf);
    fn root(&mut self) -> Self::Output;
}

impl<M: MerkleTree<H>, H: NamespaceMerkleHasher> NMT<M, H> {
    fn push_leaf(&mut self, leaf: M::Leaf, namespace: NamespaceId) {
        if namespace < self.max_ns {
            panic!("Leaves must be pushed in order");
        }

        self.inner.push_leaf(leaf);
        self.max_ns = namespace;
    }

    fn root(&mut self) -> M::Output {
        self.inner.root()
    }
}

impl<H, Db> MerkleTree<H> for SimpleMerkle<Db, H>
where
    H: NamespaceMerkleHasher,
    Db: PreimageDb<H::Output>,
{
    type Output = H::Output;
    type Leaf = Vec<u8>;

    fn root(&mut self) -> Self::Output {
        SimpleMerkle::root(self)
    }

    fn push_leaf(&mut self, leaf: Self::Leaf) {
        SimpleMerkle::push_leaf(self, leaf.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use crate::{simple_merkle::db::MemDb, NamespacedHash, NAMESPACE_ID_LEN};

    use super::*;
    #[test]
    fn test() {
        let mut nmt: NMT<
            SimpleMerkle<MemDb<NamespacedHash>, NamespacedSha2Hasher>,
            NamespacedSha2Hasher,
        > = NMT {
            inner: SimpleMerkle::default(),
            max_ns: NamespaceId([0u8; NAMESPACE_ID_LEN]),
            phantom: PhantomData::<NamespacedSha2Hasher>,
        };

        nmt.push_leaf(
            vec![1, 2, 3, 4, 5, 6, 7, 8],
            NamespaceId([1; NAMESPACE_ID_LEN]),
        );
        nmt.push_leaf(
            vec![1, 2, 3, 4, 5, 6, 7, 8],
            NamespaceId([1; NAMESPACE_ID_LEN]),
        );
        dbg!(nmt.root());
    }
}
