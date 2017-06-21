// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use err::{self, PyResult};
use ffi::{self, Py_ssize_t};
use token::PyObjectWithToken;
use pointers::PyPtr;
use python::{Python, ToPyPointer, IntoPyPointer};
use objects::PyObject;
use conversion::{ToPyObject, IntoPyObject};

/// Represents a Python `list`.
pub struct PyList(PyPtr);

pyobject_nativetype2!(PyList, PyList_Type, PyList_Check);

impl PyList {
    /// Construct a new list with the given elements.
    pub fn new<'p, T: ToPyObject>(py: Python<'p>, elements: &[T]) -> &'p PyList {
        unsafe {
            let ptr = ffi::PyList_New(elements.len() as Py_ssize_t);
            for (i, e) in elements.iter().enumerate() {
                let obj = e.to_object(py).into_ptr();
                ffi::PyList_SetItem(ptr, i as Py_ssize_t, obj);
            }
            py.unchecked_cast_from_ptr::<PyList>(ptr)
        }
    }

    /// Construct a new empty list.
    pub fn empty<'p>(py: Python<'p>) -> &'p PyList {
        unsafe {
            py.unchecked_cast_from_ptr::<PyList>(ffi::PyList_New(0))
        }
    }

    /// Gets the length of the list.
    #[inline]
    pub fn len(&self) -> usize {
        // non-negative Py_ssize_t should always fit into Rust usize
        unsafe {
            ffi::PyList_Size(self.as_ptr()) as usize
        }
    }

    /// Gets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn get_item(&self, index: isize) -> &PyObject {
        unsafe {
            let ptr = ffi::PyList_GetItem(self.as_ptr(), index as Py_ssize_t);
            let ob = PyObject::from_borrowed_ptr(self.token(), ptr);
            self.token().track_object(ob)
        }
    }

    /// Gets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn get_parked_item(&self, index: isize) -> PyObject {
        unsafe {
            let ptr = ffi::PyList_GetItem(self.as_ptr(), index as Py_ssize_t);
            PyObject::from_borrowed_ptr(self.token(), ptr)
        }
    }

    /// Sets the item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn set_item<I>(&self, index: isize, item: I) -> PyResult<()>
        where I: ToPyObject
    {
        item.with_borrowed_ptr(self.token(), |item| unsafe {
            err::error_on_minusone(
                self.token(), ffi::PyList_SetItem(self.as_ptr(), index, item))
        })
    }

    /// Inserts an item at the specified index.
    ///
    /// Panics if the index is out of range.
    pub fn insert_item<I>(&self, index: isize, item: I) -> PyResult<()>
        where I: ToPyObject
    {
        item.with_borrowed_ptr(self.token(), |item| unsafe {
            err::error_on_minusone(
                self.token(), ffi::PyList_Insert(self.as_ptr(), index, item))
        })
    }

    #[inline]
    pub fn iter(&self) -> PyListIterator {
        PyListIterator { list: self, index: 0 }
    }
}

/// Used by `PyList::iter()`.
pub struct PyListIterator<'a> {
    list: &'a PyList,
    index: isize,
}

impl<'a> Iterator for PyListIterator<'a> {
    type Item = &'a PyObject;

    #[inline]
    fn next(&mut self) -> Option<&'a PyObject> {
        if self.index < self.list.len() as isize {
            let item = self.list.get_item(self.index);
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }

    // Note: we cannot implement size_hint because the length of the list
    // might change during the iteration.
}

impl <T> ToPyObject for [T] where T: ToPyObject {

    fn to_object<'p>(&self, py: Python<'p>) -> PyObject {
        unsafe {
            let ptr = ffi::PyList_New(self.len() as Py_ssize_t);
            for (i, e) in self.iter().enumerate() {
                let obj = e.to_object(py).into_ptr();
                ffi::PyList_SetItem(ptr, i as Py_ssize_t, obj);
            }
            PyObject::from_owned_ptr_or_panic(py, ptr)
        }
    }
}

impl <T> ToPyObject for Vec<T> where T: ToPyObject {

    fn to_object<'p>(&self, py: Python<'p>) -> PyObject {
        self.as_slice().to_object(py)
    }

}

impl <T> IntoPyObject for Vec<T> where T: IntoPyObject {

    fn into_object(self, py: Python) -> PyObject {
        unsafe {
            let ptr = ffi::PyList_New(self.len() as Py_ssize_t);
            for (i, e) in self.into_iter().enumerate() {
                let obj = e.into_object(py).into_ptr();
                ffi::PyList_SetItem(ptr, i as Py_ssize_t, obj);
            }
            ::PyObject::from_owned_ptr_or_panic(py, ptr)
        }
    }
}

#[cfg(test)]
mod test {
    use python::{Python, PyDowncastFrom};
    use conversion::ToPyObject;
    use objects::PyList;

    #[test]
    fn test_new() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let list = PyList::new(py, &v);
        assert_eq!(2, list.get_item(0).extract::<i32>(py).unwrap());
        assert_eq!(3, list.get_item(1).extract::<i32>(py).unwrap());
        assert_eq!(5, list.get_item(2).extract::<i32>(py).unwrap());
        assert_eq!(7, list.get_item(3).extract::<i32>(py).unwrap());
    }

    #[test]
    fn test_len() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![1,2,3,4];
        let ob = v.to_object(py);
        let list = PyList::downcast_from(py, &ob).unwrap();
        assert_eq!(4, list.len());
    }

    #[test]
    fn test_get_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let ob = v.to_object(py);
        let list = PyList::downcast_from(py, &ob).unwrap();
        assert_eq!(2, list.get_item(0).extract::<i32>(py).unwrap());
        assert_eq!(3, list.get_item(1).extract::<i32>(py).unwrap());
        assert_eq!(5, list.get_item(2).extract::<i32>(py).unwrap());
        assert_eq!(7, list.get_item(3).extract::<i32>(py).unwrap());
    }

    #[test]
    fn test_get_parked_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let ob = v.to_object(py);
        let list = PyList::downcast_from(py, &ob).unwrap();
        assert_eq!(2, list.get_parked_item(0).extract::<i32>(py).unwrap());
        assert_eq!(3, list.get_parked_item(1).extract::<i32>(py).unwrap());
        assert_eq!(5, list.get_parked_item(2).extract::<i32>(py).unwrap());
        assert_eq!(7, list.get_parked_item(3).extract::<i32>(py).unwrap());
    }

    #[test]
    fn test_set_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let ob = v.to_object(py);
        let list = PyList::downcast_from(py, &ob).unwrap();
        let val = 42i32.to_object(py);
        assert_eq!(2, list.get_item(0).extract::<i32>(py).unwrap());
        list.set_item(0, val).unwrap();
        assert_eq!(42, list.get_item(0).extract::<i32>(py).unwrap());
    }

    #[test]
    fn test_insert_item() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let ob = v.to_object(py);
        let list = PyList::downcast_from(py, &ob).unwrap();
        let val = 42i32.to_object(py);
        assert_eq!(4, list.len());
        assert_eq!(2, list.get_item(0).extract::<i32>(py).unwrap());
        list.insert_item(0, val).unwrap();
        assert_eq!(5, list.len());
        assert_eq!(42, list.get_item(0).extract::<i32>(py).unwrap());
        assert_eq!(2, list.get_item(1).extract::<i32>(py).unwrap());
    }

    #[test]
    fn test_iter() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let ob = v.to_object(py);
        let list = PyList::downcast_from(py, &ob).unwrap();
        let mut idx = 0;
        for el in list.iter() {
            assert_eq!(v[idx], el.extract::<i32>(py).unwrap());
            idx += 1;
        }
        assert_eq!(idx, v.len());
    }

    #[test]
    fn test_extract() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let v = vec![2, 3, 5, 7];
        let ob = v.to_object(py);
        let list = PyList::downcast_from(py, &ob).unwrap();
        let v2 = list.as_ref().extract::<Vec<i32>>(py).unwrap();
        assert_eq!(v, v2);
    }
}
