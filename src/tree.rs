
pub struct TreeNode<T : Sized + Clone> {
    pub value: T,
    pub children: Vec<TreeNode<T>>,
}

