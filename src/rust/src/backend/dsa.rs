// This file is dual licensed under the terms of the Apache License, Version
// 2.0, and the BSD License. See the LICENSE file in the root of this repository
// for complete details.

use crate::backend::utils;
use crate::error::{CryptographyError, CryptographyResult};
use crate::{exceptions, types};

#[pyo3::prelude::pyclass(
    frozen,
    module = "cryptography.hazmat.bindings._rust.openssl.dsa",
    name = "DSAPrivateKey"
)]
pub(crate) struct DsaPrivateKey {
    pkey: openssl::pkey::PKey<openssl::pkey::Private>,
}

#[pyo3::prelude::pyclass(
    frozen,
    module = "cryptography.hazmat.bindings._rust.openssl.dsa",
    name = "DSAPublicKey"
)]
pub(crate) struct DsaPublicKey {
    pkey: openssl::pkey::PKey<openssl::pkey::Public>,
}

#[pyo3::prelude::pyclass(
    frozen,
    module = "cryptography.hazmat.bindings._rust.openssl.dsa",
    name = "DSAParameters"
)]
struct DsaParameters {
    dsa: openssl::dsa::Dsa<openssl::pkey::Params>,
}

pub(crate) fn private_key_from_pkey(
    pkey: &openssl::pkey::PKeyRef<openssl::pkey::Private>,
) -> DsaPrivateKey {
    DsaPrivateKey {
        pkey: pkey.to_owned(),
    }
}

pub(crate) fn public_key_from_pkey(
    pkey: &openssl::pkey::PKeyRef<openssl::pkey::Public>,
) -> DsaPublicKey {
    DsaPublicKey {
        pkey: pkey.to_owned(),
    }
}

#[pyo3::prelude::pyfunction]
fn generate_parameters(key_size: u32) -> CryptographyResult<DsaParameters> {
    let dsa = openssl::dsa::Dsa::generate_params(key_size)?;
    Ok(DsaParameters { dsa })
}

#[pyo3::prelude::pyfunction]
fn from_private_numbers(
    py: pyo3::Python<'_>,
    numbers: &pyo3::PyAny,
) -> CryptographyResult<DsaPrivateKey> {
    let public_numbers = numbers.getattr(pyo3::intern!(py, "public_numbers"))?;
    let parameter_numbers = public_numbers.getattr(pyo3::intern!(py, "parameter_numbers"))?;

    let dsa = openssl::dsa::Dsa::from_private_components(
        utils::py_int_to_bn(py, parameter_numbers.getattr(pyo3::intern!(py, "p"))?)?,
        utils::py_int_to_bn(py, parameter_numbers.getattr(pyo3::intern!(py, "q"))?)?,
        utils::py_int_to_bn(py, parameter_numbers.getattr(pyo3::intern!(py, "g"))?)?,
        utils::py_int_to_bn(py, numbers.getattr(pyo3::intern!(py, "x"))?)?,
        utils::py_int_to_bn(py, public_numbers.getattr(pyo3::intern!(py, "y"))?)?,
    )
    .unwrap();
    let pkey = openssl::pkey::PKey::from_dsa(dsa)?;
    Ok(DsaPrivateKey { pkey })
}

#[pyo3::prelude::pyfunction]
fn from_public_numbers(
    py: pyo3::Python<'_>,
    numbers: &pyo3::PyAny,
) -> CryptographyResult<DsaPublicKey> {
    let parameter_numbers = numbers.getattr(pyo3::intern!(py, "parameter_numbers"))?;

    let dsa = openssl::dsa::Dsa::from_public_components(
        utils::py_int_to_bn(py, parameter_numbers.getattr(pyo3::intern!(py, "p"))?)?,
        utils::py_int_to_bn(py, parameter_numbers.getattr(pyo3::intern!(py, "q"))?)?,
        utils::py_int_to_bn(py, parameter_numbers.getattr(pyo3::intern!(py, "g"))?)?,
        utils::py_int_to_bn(py, numbers.getattr(pyo3::intern!(py, "y"))?)?,
    )
    .unwrap();
    let pkey = openssl::pkey::PKey::from_dsa(dsa)?;
    Ok(DsaPublicKey { pkey })
}

#[pyo3::prelude::pyfunction]
fn from_parameter_numbers(
    py: pyo3::Python<'_>,
    numbers: &pyo3::PyAny,
) -> CryptographyResult<DsaParameters> {
    let dsa = openssl::dsa::Dsa::from_pqg(
        utils::py_int_to_bn(py, numbers.getattr(pyo3::intern!(py, "p"))?)?,
        utils::py_int_to_bn(py, numbers.getattr(pyo3::intern!(py, "q"))?)?,
        utils::py_int_to_bn(py, numbers.getattr(pyo3::intern!(py, "g"))?)?,
    )
    .unwrap();
    Ok(DsaParameters { dsa })
}

fn clone_dsa_params<T: openssl::pkey::HasParams>(
    d: &openssl::dsa::Dsa<T>,
) -> Result<openssl::dsa::Dsa<openssl::pkey::Params>, openssl::error::ErrorStack> {
    openssl::dsa::Dsa::from_pqg(d.p().to_owned()?, d.q().to_owned()?, d.g().to_owned()?)
}

#[pyo3::prelude::pymethods]
impl DsaPrivateKey {
    fn sign<'p>(
        &self,
        py: pyo3::Python<'p>,
        data: &[u8],
        algorithm: &pyo3::PyAny,
    ) -> CryptographyResult<&'p pyo3::types::PyBytes> {
        let (data, _) = utils::calculate_digest_and_algorithm(py, data, algorithm)?;

        let mut signer = openssl::pkey_ctx::PkeyCtx::new(&self.pkey)?;
        signer.sign_init()?;
        let mut sig = vec![];
        signer.sign_to_vec(data, &mut sig)?;
        Ok(pyo3::types::PyBytes::new(py, &sig))
    }

    #[getter]
    fn key_size(&self) -> i32 {
        self.pkey.dsa().unwrap().p().num_bits()
    }

    fn public_key(&self) -> CryptographyResult<DsaPublicKey> {
        let priv_dsa = self.pkey.dsa()?;
        let pub_dsa = openssl::dsa::Dsa::from_public_components(
            priv_dsa.p().to_owned()?,
            priv_dsa.q().to_owned()?,
            priv_dsa.g().to_owned()?,
            priv_dsa.pub_key().to_owned()?,
        )
        .unwrap();
        let pkey = openssl::pkey::PKey::from_dsa(pub_dsa)?;
        Ok(DsaPublicKey { pkey })
    }

    fn parameters(&self) -> CryptographyResult<DsaParameters> {
        let dsa = clone_dsa_params(&self.pkey.dsa().unwrap())?;
        Ok(DsaParameters { dsa })
    }

    fn private_numbers<'p>(&self, py: pyo3::Python<'p>) -> CryptographyResult<&'p pyo3::PyAny> {
        let dsa = self.pkey.dsa().unwrap();

        let py_p = utils::bn_to_py_int(py, dsa.p())?;
        let py_q = utils::bn_to_py_int(py, dsa.q())?;
        let py_g = utils::bn_to_py_int(py, dsa.g())?;

        let py_pub_key = utils::bn_to_py_int(py, dsa.pub_key())?;
        let py_private_key = utils::bn_to_py_int(py, dsa.priv_key())?;

        let parameter_numbers = types::DSA_PARAMETER_NUMBERS
            .get(py)?
            .call1((py_p, py_q, py_g))?;
        let public_numbers = types::DSA_PUBLIC_NUMBERS
            .get(py)?
            .call1((py_pub_key, parameter_numbers))?;

        Ok(types::DSA_PRIVATE_NUMBERS
            .get(py)?
            .call1((py_private_key, public_numbers))?)
    }

    fn private_bytes<'p>(
        slf: &pyo3::PyCell<Self>,
        py: pyo3::Python<'p>,
        encoding: &pyo3::PyAny,
        format: &pyo3::PyAny,
        encryption_algorithm: &pyo3::PyAny,
    ) -> CryptographyResult<&'p pyo3::types::PyBytes> {
        utils::pkey_private_bytes(
            py,
            slf,
            &slf.borrow().pkey,
            encoding,
            format,
            encryption_algorithm,
            true,
            false,
        )
    }
}

#[pyo3::prelude::pymethods]
impl DsaPublicKey {
    fn verify(
        &self,
        py: pyo3::Python<'_>,
        signature: &[u8],
        data: &[u8],
        algorithm: &pyo3::PyAny,
    ) -> CryptographyResult<()> {
        let (data, _) = utils::calculate_digest_and_algorithm(py, data, algorithm)?;

        let mut verifier = openssl::pkey_ctx::PkeyCtx::new(&self.pkey)?;
        verifier.verify_init()?;
        let valid = verifier.verify(data, signature).unwrap_or(false);
        if !valid {
            return Err(CryptographyError::from(
                exceptions::InvalidSignature::new_err(()),
            ));
        }

        Ok(())
    }

    #[getter]
    fn key_size(&self) -> i32 {
        self.pkey.dsa().unwrap().p().num_bits()
    }

    fn parameters(&self) -> CryptographyResult<DsaParameters> {
        let dsa = clone_dsa_params(&self.pkey.dsa().unwrap())?;
        Ok(DsaParameters { dsa })
    }

    fn public_numbers<'p>(&self, py: pyo3::Python<'p>) -> CryptographyResult<&'p pyo3::PyAny> {
        let dsa = self.pkey.dsa().unwrap();

        let py_p = utils::bn_to_py_int(py, dsa.p())?;
        let py_q = utils::bn_to_py_int(py, dsa.q())?;
        let py_g = utils::bn_to_py_int(py, dsa.g())?;

        let py_pub_key = utils::bn_to_py_int(py, dsa.pub_key())?;

        let parameter_numbers = types::DSA_PARAMETER_NUMBERS
            .get(py)?
            .call1((py_p, py_q, py_g))?;
        Ok(types::DSA_PUBLIC_NUMBERS
            .get(py)?
            .call1((py_pub_key, parameter_numbers))?)
    }

    fn public_bytes<'p>(
        slf: &pyo3::PyCell<Self>,
        py: pyo3::Python<'p>,
        encoding: &pyo3::PyAny,
        format: &pyo3::PyAny,
    ) -> CryptographyResult<&'p pyo3::types::PyBytes> {
        utils::pkey_public_bytes(py, slf, &slf.borrow().pkey, encoding, format, true, false)
    }

    fn __eq__(&self, other: pyo3::PyRef<'_, Self>) -> bool {
        self.pkey.public_eq(&other.pkey)
    }

    fn __copy__(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        slf
    }
}

#[pyo3::prelude::pymethods]
impl DsaParameters {
    fn generate_private_key(&self) -> CryptographyResult<DsaPrivateKey> {
        let dsa = clone_dsa_params(&self.dsa)?.generate_key()?;
        let pkey = openssl::pkey::PKey::from_dsa(dsa)?;
        Ok(DsaPrivateKey { pkey })
    }

    fn parameter_numbers<'p>(&self, py: pyo3::Python<'p>) -> CryptographyResult<&'p pyo3::PyAny> {
        let py_p = utils::bn_to_py_int(py, self.dsa.p())?;
        let py_q = utils::bn_to_py_int(py, self.dsa.q())?;
        let py_g = utils::bn_to_py_int(py, self.dsa.g())?;

        Ok(types::DSA_PARAMETER_NUMBERS
            .get(py)?
            .call1((py_p, py_q, py_g))?)
    }
}

pub(crate) fn create_module(py: pyo3::Python<'_>) -> pyo3::PyResult<&pyo3::prelude::PyModule> {
    let m = pyo3::prelude::PyModule::new(py, "dsa")?;
    m.add_function(pyo3::wrap_pyfunction!(generate_parameters, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(from_private_numbers, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(from_public_numbers, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(from_parameter_numbers, m)?)?;

    m.add_class::<DsaPrivateKey>()?;
    m.add_class::<DsaPublicKey>()?;
    m.add_class::<DsaParameters>()?;

    Ok(m)
}
