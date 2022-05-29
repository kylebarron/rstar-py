use itertools::izip;
use numpy::PyReadonlyArrayDyn;
use pyo3::prelude::*;
use rstar::primitives::{GeomWithData, Rectangle};

// type
type TreeGeometryType = Rectangle<[f64; 2]>;
type TreeItemType = GeomWithData<TreeGeometryType, usize>;
type Tree2DType = rstar::RTree<TreeItemType>;

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
}
