/* Copyright 2024 Canonical Ltd.
 *
 * This program is free software: you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public License
 * version 3, as published by the Free Software Foundation.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * Written by:
 *        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
 */

use crate::{
    models::request_validators::{CertificationStatusRequest, Paths},
    send_certification_status_request as native_send_certification_status_request,
};
use pyo3::{
    exceptions::PyRuntimeError, prelude::*, types::PyString, wrap_pyfunction, Py, PyAny, PyResult,
    Python,
};
use serde::Serialize;
use serde_json;

/// Helper function to convert a Rust serializable value to a Python object.
///
/// This function serializes the Rust value to JSON and then converts it to a
/// Python object using Python's json module.
///
/// # Arguments
/// * `py` - Python GIL token
/// * `value` - Any Rust value that implements Serialize
///
/// # Returns
/// A Python object representing the JSON-serialized value
///
/// # Errors
/// Returns `PyErr` if:
/// - JSON serialization fails
/// - Python json module cannot be imported
/// - Python json.loads fails to parse the JSON string
fn to_python_json(py: Python, value: impl Serialize) -> PyResult<Py<PyAny>> {
    let json_str = serde_json::json!(value).to_string();
    let json = PyString::new(py, &json_str);
    let json_module = py.import("json")?;
    let json_object: Py<PyAny> = json_module.call_method1("loads", (json,))?.into();
    Ok(json_object)
}

/// Helper function to create a certification status request.
///
/// This function creates a `CertificationStatusRequest` using default system paths.
///
/// # Returns
/// A `CertificationStatusRequest` containing hardware and system information
///
/// # Errors
/// Returns `PyErr` if:
/// - Hardware information cannot be collected from the system
/// - Required system files are missing or inaccessible
/// - Data parsing or validation fails
fn create_certification_request() -> PyResult<CertificationStatusRequest> {
    CertificationStatusRequest::new(Paths::default())
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to create request: {e}")))
}

/// This function creates and sends the certification status request to the specified
/// hardware-api server URL.
///
/// The function performs the following steps:
/// 1. Collects hardware and system information
/// 2. Creates a certification status request
/// 3. Sends the request to the specified server URL
/// 4. Converts the response to a Python object
///
/// # Arguments
/// * `py` - Python GIL token
/// * `url` - Base URL of the hardware-api server (e.g., "https://api.example.com")
///
/// # Returns
/// A Python dictionary containing the certification status response
///
/// # Errors
/// Returns `PyErr` if:
/// - Hardware information collection fails (e.g., missing system files, permission errors)
/// - Request creation fails due to data validation errors
/// - Network request fails (e.g., connection timeout, invalid URL)
/// - Server returns an error response
/// - Response cannot be converted to Python object
///
/// # Example
/// ```python
/// import hwlib
/// response = hwlib.send_certification_request("https://hardware-api.example.com")
/// print(response["status"])
/// ```
#[pyfunction]
fn send_certification_request(py: Python, url: String) -> PyResult<Py<PyAny>> {
    let request_body = create_certification_request()?;

    let response = native_send_certification_status_request(url, &request_body)
        .map_err(|e| PyRuntimeError::new_err(format!("Request failed: {e}")))?;

    to_python_json(py, response)
}

#[pymodule]
fn hwlib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(send_certification_request, m)?)?;
    Ok(())
}

// Note: Unit tests for the helper functions would require Python runtime to be available.
// The functions are integration tested through the existing Python test in pytests/test_cert_status.py
// which validates that send_certification_request can be imported and called successfully.
