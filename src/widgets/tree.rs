use crate::ext::{DisplayStr, LayoutWithOptions};
use crate::widgets::TreeDecoration;
use crate::widgets::tree::formatted::FormattedTreeNode;
use crate::widgets::vertical::FormattedVertical;
use crate::{BoxedFormattedLayout, Dimension, Layout, LayoutOptions, RcLayout, Rect, WrapMode};
use std::any::Any;
use std::cmp::max;

pub(crate) mod decoration;
mod formatted;

/// A widget for displaying hierarchical tree structures.
///
/// The data of a `Tree` is composed of [`TreeNode`]s, where each node contains a [`RcLayout`]
/// representing the node's content and a list of child nodes.
/// The tree visualization can be customized using a [`TreeDecoration`].
///
/// # Example
/// ```rust
/// use termlayout::*;
/// use termlayout::widgets::{Paragraph, Tree, TreeDecoration, TreeNode};
///
/// let tree = Tree::new(TreeDecoration::default(),
///     TreeNode::new(Paragraph::left("A tree is composed of TreeNodes. This is the root item"), vec![
///     Paragraph::left("A TreeNode might be a simple RcLayout like this one").into(),
///     TreeNode::new(Paragraph::left("Or a complex one with children:"), vec![
///         Paragraph::left("child1"),
///         Paragraph::left("child2")
///     ])
/// ]), true);
///
/// assert_eq!(format!("{}", tree.layout(30)), concat!(
///     "A tree is composed of\n",
///     "TreeNodes. This is the root\n",
///     "item\n",
///     "├─ A TreeNode might be a\n",
///     "│  simple RcLayout like this\n",
///     "│  one\n",
///     "└─ Or a complex one with\n",
///     "   children:\n",
///     "   ├─ child1\n",
///     "   └─ child2\n"
/// ));
/// ```
pub struct Tree {
    /// The [`TreeDecoration`] used to draw the tree.
    pub decoration: TreeDecoration,

    /// The root node of the tree.
    pub root: TreeNode,

    /// A boolean flag indicating whether the root node should be displayed.
    pub show_root: bool,
}

impl Tree {
    /// Creates a new [`Tree`] instance with the given decoration, root node, and root visibility
    /// flag.
    ///
    /// # Parameters
    /// - `decoration`: The [`TreeDecoration`] to use
    /// - `root`: The root node of the tree
    /// - `show_root`: A boolean flag indicating whether the root node should be displayed
    ///
    /// # Returns
    /// A new [`Tree`] instance with the provided configuration
    ///
    /// # Example
    /// ```rust
    /// use termlayout::*;
    /// use termlayout::widgets::{Lines, Tree, TreeDecoration, TreeNode};
    ///
    /// let tree = Tree::new(TreeDecoration::default(), TreeNode::new(Lines::left("root"), vec![
    ///     Lines::left("child1"),
    ///     Lines::left("child2"),
    /// ]), true);
    ///
    /// assert_eq!(format!("{}", tree.layout(20)), concat!(
    ///     "root\n",
    ///     "├─ child1\n",
    ///     "└─ child2\n"
    /// ));
    /// ```
    pub fn new<T>(decoration: TreeDecoration, root: T, show_root: bool) -> Self
    where
        T: Into<TreeNode>,
    {
        Self {
            decoration,
            root: root.into(),
            show_root,
        }
    }

    fn format_node(
        &self,
        path: &TreePath,
        options: &LayoutOptions,
        offset: usize,
    ) -> BoxedFormattedLayout<'static> {
        let prefix = path.prefixes(&self.decoration);
        let available_width = max(1, options.dim.width.saturating_sub(prefix.0.display_len()));
        let prefix_len = options.dim.width.saturating_sub(available_width);
        let pref_dim = path.node.item.pref_dim(available_width, options.wrap_mode);
        let item_opts = options.intersect(Rect::new(prefix_len, offset, pref_dim), false);
        let item = LayoutWithOptions::of(path.node.item.clone(), item_opts).into();
        let node_opts = options.intersect(
            Rect::new(
                0,
                offset,
                Dimension::new(options.dim.width, pref_dim.height),
            ),
            false,
        );
        let prefix = (
            prefix.0.display_slice(0..prefix_len).to_string(),
            prefix.1.display_slice(0..prefix_len).to_string(),
        );
        FormattedTreeNode::new(prefix, item, node_opts).into()
    }
}

impl Layout for Tree {
    fn pref_dim(&self, max_width: usize, wrap_mode: WrapMode) -> Dimension {
        if max_width == 0 {
            return Dimension::empty();
        }
        let mut dim = Dimension::empty();
        let prefix_len = self.decoration.prefix_len();
        self.root.traverse(self.show_root, |path| {
            let available_width = max(1, max_width.saturating_sub(prefix_len * path.depth));
            let mut node_dim = path.node.item.pref_dim(available_width, wrap_mode);
            node_dim.width += prefix_len * path.depth;
            dim = dim.vertical_union(node_dim);
        });
        dim
    }

    fn min_dim(&self) -> Dimension {
        let mut dim = Dimension::empty();
        let prefix_len = self.decoration.prefix_len();
        self.root.traverse(self.show_root, |path| {
            let mut node_dim = path.node.item.min_dim();
            node_dim.width += prefix_len * path.depth;
            dim = dim.vertical_union(node_dim);
        });
        dim
    }

    fn layout_strict(&'_ self, options: LayoutOptions) -> BoxedFormattedLayout<'_> {
        let mut rows = vec![];
        let mut offset = 0;
        self.root.traverse(self.show_root, |path| {
            let node = self.format_node(path, &options, offset);
            offset += node.options().dim.height;
            rows.push(node);
        });

        if rows.len() == 1 {
            return rows.remove(0);
        }

        FormattedVertical::new(rows, options.with_normalized_horizontal_clip()).into()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Represents a node in a [`Tree`].
/// A `TreeNode` has a [`RcLayout`] representing the node text and a list of [`TreeNode`]s representing
/// the children
pub struct TreeNode {
    /// The tree item node
    pub item: RcLayout,

    /// The list of children
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    /// Creates a new [`TreeNode`].
    ///
    /// # Parameters
    /// * `item` - The item representing the node text
    /// * `children` - A list of child nodes
    ///
    /// # Returns
    /// A new instance
    pub fn new<I, C>(item: I, children: Vec<C>) -> Self
    where
        I: Into<RcLayout>,
        C: Into<TreeNode>,
    {
        Self {
            item: item.into(),
            children: children.into_iter().map(Into::into).collect(),
        }
    }

    /// Creates a new leaf [`TreeNode`].
    /// The node initially has no children.
    ///
    /// # Parameters
    /// * `item` - The item representing the node text
    ///
    /// # Returns
    /// A new instance
    ///
    /// # Example
    /// ```rust
    ///
    /// use termlayout::widgets::{Lines, TreeNode};
    /// let node = TreeNode::leaf(Lines::left("The node text"));
    ///
    /// assert_eq!(node.children.len(), 0)
    /// ```
    pub fn leaf<I>(item: I) -> Self
    where
        I: Into<RcLayout>,
    {
        Self {
            item: item.into(),
            children: Vec::new(),
        }
    }

    /// Traverses the tree structure starting from the current node and applies the provided
    /// callback function to each node.
    ///
    /// # Parameters
    /// - `include_self`: A boolean flag indicating whether the current node (`self`) should be
    ///   included in the traversal. If `true`, the callback will also be invoked for the current
    ///   node.
    /// - `callback`: A closure or function that takes a reference to a [`TreePath`] and performs
    ///   an operation on it. This closure must implement the `FnMut` trait, allowing it to have
    ///   mutable state.
    pub fn traverse<T>(&self, include_self: bool, mut callback: T)
    where
        T: FnMut(&TreePath),
    {
        let path = TreePath::new(self, true, None);
        if include_self {
            callback(&path);
        }
        path.traverse_children(&mut callback);
    }
}

impl<T> From<T> for TreeNode
where
    T: Into<RcLayout>,
{
    fn from(item: T) -> Self {
        Self::leaf(item)
    }
}

/// Represents the path taken to a node in a [`Tree`] structure.
/// This structure is important when traversing through a tree using
/// [`TreeNode::traverse_children()`](TreeNode::traverse) and contains further information about the
/// node's position in the tree.
pub struct TreePath<'a> {
    /// The parent path, if any
    pub parent: Option<&'a TreePath<'a>>,

    /// The current node
    pub node: &'a TreeNode,

    /// A boolean flag indicating whether the current node is the last child of its parent
    pub last_child: bool,

    /// The depth of the node in the tree
    pub depth: usize,
}

impl<'a> TreePath<'a> {
    fn new(node: &'a TreeNode, last_child: bool, parent: Option<&'a TreePath<'a>>) -> Self {
        Self {
            parent,
            node,
            last_child,
            depth: parent.map_or(0, |p| p.depth + 1),
        }
    }

    fn traverse_children<T>(&self, callback: &mut T)
    where
        T: FnMut(&TreePath),
    {
        let len = self.node.children.len();
        for (index, child) in self.node.children.iter().enumerate() {
            let child = TreePath::new(child, index + 1 == len, Some(self));
            callback(&child);
            child.traverse_children(callback);
        }
    }

    fn prefixes(&self, decoration: &TreeDecoration) -> (String, String) {
        if let Some(parent) = self.parent {
            let (_, parent_next) = parent.prefixes(decoration);
            let (first, next) = decoration.prefixes(self.last_child);
            (parent_next.clone() + first, parent_next + next)
        } else {
            (String::new(), String::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::Lines;

    fn sample_nodes() -> TreeNode {
        TreeNode::new(
            Lines::left("root\nLevel 0"),
            vec![
                TreeNode::new(
                    Lines::left("child1\nLevel 1"),
                    vec![
                        TreeNode::new(
                            Lines::left("child1.1\nLevel 2"),
                            vec![
                                TreeNode::leaf(Lines::left("child1.1.1\nLevel 3")),
                                TreeNode::leaf(Lines::left("child1.1.2\nLevel 3")),
                            ],
                        ),
                        TreeNode::leaf(Lines::left("child1.2\nLevel 2")),
                    ],
                ),
                TreeNode::new(
                    Lines::left("child2\nLevel 1"),
                    vec![
                        TreeNode::leaf(Lines::left("child2.1\nLevel 2")),
                        TreeNode::new(
                            Lines::left("child2.2\nLevel 2"),
                            vec![
                                TreeNode::leaf(Lines::left("child2.2.1\nLevel 3")),
                                TreeNode::leaf(Lines::left("child2.2.2\nLevel 3")),
                            ],
                        ),
                    ],
                ),
            ],
        )
    }

    #[test]
    fn tree_layout_no_clip() {
        let tree = Tree::new(TreeDecoration::lines(1), sample_nodes(), true);

        let formatted = tree.layout_strict(LayoutOptions::new(
            Dimension::new(20, 23),
            true,
            WrapMode::Wrap,
            None,
        ));

        assert_eq!(
            format!("{formatted}"),
            concat!(
                "root                \n",
                "Level 0             \n",
                "├─ child1           \n",
                "│  Level 1          \n",
                "│  ├─ child1.1      \n",
                "│  │  Level 2       \n",
                "│  │  ├─ child1.1.1 \n",
                "│  │  │  Level 3    \n",
                "│  │  └─ child1.1.2 \n",
                "│  │     Level 3    \n",
                "│  └─ child1.2      \n",
                "│     Level 2       \n",
                "└─ child2           \n",
                "   Level 1          \n",
                "   ├─ child2.1      \n",
                "   │  Level 2       \n",
                "   └─ child2.2      \n",
                "      Level 2       \n",
                "      ├─ child2.2.1 \n",
                "      │  Level 3    \n",
                "      └─ child2.2.2 \n",
                "         Level 3    \n",
                "                    \n",
            )
        );
    }

    #[test]
    fn tree_layout_with_clip() {
        let tree = Tree::new(TreeDecoration::lines(1), sample_nodes(), true);

        let formatted = tree.layout_strict(LayoutOptions::new(
            Dimension::new(20, 23),
            true,
            WrapMode::Wrap,
            Some(Rect::new(1, 2, Dimension::new(11, 13))),
        ));

        assert_eq!(
            format!("{formatted}"),
            concat!(
                "─ child1   \n",
                "  Level 1  \n",
                "  ├─ child1\n",
                "  │  Level \n",
                "  │  ├─ chi\n",
                "  │  │  Lev\n",
                "  │  └─ chi\n",
                "  │     Lev\n",
                "  └─ child1\n",
                "     Level \n",
                "─ child2   \n",
                "  Level 1  \n",
                "  ├─ child2\n"
            )
        );
    }

    #[test]
    fn tree_layout_zero_width() {
        // Arrange
        let tree = Tree::new(TreeDecoration::lines(1), sample_nodes(), true);

        assert_eq!(format!("{}", tree.layout(0)), "");
    }
}
