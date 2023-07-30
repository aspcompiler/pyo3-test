#![allow(dead_code)]
use std::sync::{Arc, Mutex};

use pyo3::{prelude::*, exceptions::PyStopAsyncIteration};

// 1. A normal function that returns a String.

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

// 2. Return a class defined in Rust.
#[pyclass]
struct MyClass {
    #[pyo3(get, set)]
    num: i32,
}

#[pyfunction]
fn return_myclass() -> Py<MyClass> {
    Python::with_gil(|py| -> Py<MyClass> {
        Py::new(py, MyClass { num: 1 }).unwrap()
    })
}

// 3. Iterator

#[pyclass]
struct PyClassIter {
    count: usize,
}

#[pymethods]
impl PyClassIter {
    #[new]
    pub fn new() -> Self {
        PyClassIter { count: 0 }
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<usize> {
        if self.count < 5 {
            self.count += 1;
            // Given an instance `counter`, First five `next(counter)` calls yield 1, 2, 3, 4, 5.
            Some(self.count)
        } else {
            None
        }
    }
}

// 4. Async Iterator

#[pyclass]
struct PyAsyncIter {
    count: Arc<Mutex<usize>>,
}

#[pymethods]
impl PyAsyncIter {
    #[new]
    pub fn new() -> Self {
        PyAsyncIter { count: Arc::new(Mutex::new(0)) }
    }

    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__(slf: PyRefMut<'_, Self>) -> PyResult<Option<PyObject>> {
        let count = slf.count.clone();
        let fut = pyo3_asyncio::tokio::future_into_py(slf.py(), async move {
            let mut count = count.lock().unwrap();
            if *count < 5 {
                *count += 1;
                Ok(Python::with_gil(|py| count.into_py(py)))
            } else {
                Err(PyStopAsyncIteration::new_err("stream exhausted"))
            }    
        })?;
        Ok(Some(fut.into()))
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyo3_test(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;

    m.add_class::<MyClass>()?;
    m.add_function(wrap_pyfunction!(return_myclass, m)?)?;

    m.add_class::<PyClassIter>()?;

    m.add_class::<PyAsyncIter>()?;
    Ok(())
}