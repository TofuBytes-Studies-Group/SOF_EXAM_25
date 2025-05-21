use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Weapon {
    pub name: String,
    pub damage: i32,
    pub weight: f64,
    pub upgrade: String,
    pub perk: String,
    pub weapon_type: String,
    pub predicted_price: Option<f64>,
}

pub fn generate_weapon(base_name: &str) -> PyResult<Weapon> {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let sys = py.import("sys")?;
        sys.getattr("path")?
            .call_method1("append", ("./python",))?;

        let ai = py.import("ai_calls")?;
        let predictor = py.import("model_predictor")?;

        let generate_name = ai.getattr("generate_weapon_name")?;
        let full_name: String = generate_name.call1((base_name,))?.extract()?;

        let generate_stats = ai.getattr("generate_weapon")?;
        let py_result = generate_stats.call1((full_name.clone(),))?;
        let py_dict = py_result.downcast::<PyDict>()?;

        let mut weapon = Weapon {
            name: extract_string(&py_dict, "Name")?,
            damage: extract_i32(&py_dict, "Damage")?,
            weight: extract_f64(&py_dict, "Weight")?,
            upgrade: extract_string(&py_dict, "Upgrade")?,
            perk: extract_string(&py_dict, "Perk")?,
            weapon_type: extract_string(&py_dict, "Type")?,
            predicted_price: None,
        };

        let predict = predictor.getattr("predict_price")?;
        let predicted: f64 = predict.call1((py_dict,))?.extract()?;
        weapon.predicted_price = Some(predicted);

        Ok(weapon)
    })
}

fn extract_string(dict: &&Bound<PyDict>, key: &str) -> PyResult<String> {
    dict.get_item(key)?
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!("Missing key: {}", key)))?
        .extract()
}

fn extract_i32(dict: &&Bound<PyDict>, key: &str) -> PyResult<i32> {
    dict.get_item(key)?
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!("Missing key: {}", key)))?
        .extract()
}

fn extract_f64(dict: &&Bound<PyDict>, key: &str) -> PyResult<f64> {
    dict.get_item(key)?
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!("Missing key: {}", key)))?
        .extract()
}
