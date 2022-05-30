use itertools::izip;
use numpy::{IntoPyArray, Ix1, PyArray, PyReadonlyArrayDyn};
use pyo3::prelude::*;
use pyo3::types::PyType;
use rstar::primitives::{GeomWithData, Rectangle};

type PointType = [f64; 2];
type BBoxType = [f64; 4];
type TreeGeometryType = Rectangle<PointType>;
type TreeItemType = GeomWithData<TreeGeometryType, usize>;
type Tree2DType = rstar::RTree<TreeItemType>;
type ParentNodeType = rstar::ParentNode<TreeItemType>;
type AABBType = rstar::AABB<PointType>;

#[pyclass]
pub struct AABB(AABBType);

#[pymethods]
impl AABB {
    /// Get the lower left corner of the AABB
    fn ll(&self) -> PyResult<PointType> {
        Ok(self.0.lower())
    }

    /// Get the upper right corner of the AABB
    fn ur(&self) -> PyResult<PointType> {
        Ok(self.0.upper())
    }

    /// Construct an AABB from lower left and upper right corners
    #[allow(unused_variables)]
    #[classmethod]
    fn from_corners(cls: &PyType, ll: PointType, ur: PointType) -> PyResult<Self> {
        Ok(AABB(AABBType::from_corners(ll, ur)))
    }
}

#[pyclass]
pub struct Leaf(TreeItemType);

#[pymethods]
impl Leaf {
    /// Get bounding box of this leaf
    fn bbox(&self) -> PyResult<BBoxType> {
        let geom = self.0.geom();
        let mut out = BBoxType::default();
        let ll = geom.lower();
        let ur = geom.upper();
        out[0] = ll[0];
        out[1] = ll[1];
        out[2] = ur[0];
        out[3] = ur[1];

        Ok(out)
    }

    /// Get the lower left corner of this Leaf
    fn ll(&self) -> PyResult<PointType> {
        Ok(self.0.geom().lower())
    }

    /// Get the upper right corner of this Leaf
    fn ur(&self) -> PyResult<PointType> {
        Ok(self.0.geom().upper())
    }

    /// Get id of this item
    fn id(&self) -> PyResult<usize> {
        Ok(self.0.data)
    }
}

/// Get all leafs within this parent, recursively
fn get_all_leafs_of_parent(parent_node: &ParentNodeType) -> Vec<Leaf> {
    let mut output: Vec<Leaf> = Vec::new();

    for child in parent_node.children() {
        match child {
            rstar::RTreeNode::Leaf(leaf) => output.push(Leaf(*leaf)),
            rstar::RTreeNode::Parent(parent_node) => {
                output.extend(get_all_leafs_of_parent(parent_node))
            }
        }
    }

    output
}

#[pyclass]
pub struct ParentNode(ParentNodeType);

#[pymethods]
impl ParentNode {
    /// Get children of this node
    /// Returns a list of either Leaf objects or other ParentNode objects
    fn children(&self) -> PyResult<Vec<PyObject>> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let output: Vec<PyObject> = self
            .0
            .children()
            .into_iter()
            .map(|child| match child {
                rstar::RTreeNode::Leaf(leaf) => Leaf(*leaf).into_py(py),
                rstar::RTreeNode::Parent(parent_node) => {
                    ParentNode(parent_node.clone()).into_py(py)
                }
            })
            .collect();
        Ok(output)
    }

    /// Get a numpy array of all ids within this node
    fn child_ids<'py>(&self, py: Python<'py>) -> PyResult<&'py PyArray<usize, Ix1>> {
        let all_child_ids: Vec<usize> = get_all_leafs_of_parent(&self.0)
            .into_iter()
            .map(|leaf| leaf.0.data)
            .collect();

        let py_array = all_child_ids.into_pyarray(py);

        Ok(py_array)
    }

    /// Get envelope of this node
    fn aabb(&self) -> PyResult<AABB> {
        Ok(AABB(self.0.envelope()))
    }

    /// Get the lower left corner of this node
    fn ll(&self) -> PyResult<PointType> {
        Ok(self.0.envelope().lower())
    }

    /// Get the upper right corner of this node
    fn ur(&self) -> PyResult<PointType> {
        Ok(self.0.envelope().upper())
    }
}

#[pyclass]
pub struct RTree {
    tree: Tree2DType,
}

#[pymethods]
impl RTree {
    #[new]
    fn new(
        min_x: PyReadonlyArrayDyn<'_, f64>,
        min_y: PyReadonlyArrayDyn<'_, f64>,
        max_x: PyReadonlyArrayDyn<'_, f64>,
        max_y: PyReadonlyArrayDyn<'_, f64>,
    ) -> PyResult<Self> {
        let min_x = min_x.as_array();
        let min_y = min_y.as_array();
        let max_x = max_x.as_array();
        let max_y = max_y.as_array();

        let mut insertion_data: Vec<TreeItemType> = Vec::with_capacity(min_x.len());
        for (i, bbox) in izip!(&min_x, &min_y, &max_x, &max_y).enumerate() {
            let rect = TreeGeometryType::from_corners([*bbox.0, *bbox.1], [*bbox.2, *bbox.3]);
            insertion_data.push(TreeItemType::new(rect, i));
        }

        let tree = Tree2DType::bulk_load(insertion_data);

        Ok(RTree { tree })
    }

    pub fn root(&self) -> PyResult<ParentNode> {
        Ok(ParentNode(self.tree.root().clone()))
    }
}
