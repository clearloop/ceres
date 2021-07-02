//! Instantiate Entry
use crate::{contract::GasMeter, transfer::TransferEntry, Sandbox};
use ceres_executor::{Error, Executor, Memory, Result, ReturnData};
use ceres_std::Vec;
use parity_wasm::elements::Module;

/// Instantiate Entry
#[derive(Default)]
pub struct InstantiateEntry {
    pub code_hash: [u8; 32],
    pub endowment: u64,
    pub data: Vec<u8>,
    pub gas_left: u64,
    pub salt: Vec<u8>,
}

impl Sandbox {
    // Invoke (ink) method
    pub fn invoke(
        &mut self,
        code_hash: [u8; 32],
        method: &str,
        data: Vec<u8>,
    ) -> Result<([u8; 32], ReturnData)> {
        self.input = Some(data);

        // Push new frame
        let mut cache_mut = self.cache.borrow_mut();
        if cache_mut.switch(code_hash).is_none() {
            let contract = cache_mut.get(&code_hash).ok_or(Error::CodeNotFound)?;
            let limit = ceres_executor::scan_imports(&Module::from_bytes(&contract)?)?;
            let memory = Memory::new(limit.0, limit.1)?;
            cache_mut.push(code_hash, memory);
        }
        drop(cache_mut);

        // invoke with provided `data`
        let mut executor = Executor::new(code_hash, self)?;
        let ret = executor.invoke(method, &[], self)?;

        // Pop state
        self.cache.borrow_mut().back().ok_or(Error::StateNotFound)?;

        // return vals
        Ok((code_hash, ret.data))
    }

    pub fn instantiate(
        &mut self,
        code_hash: [u8; 32],
        // endowment: u64,
        gas_meter: &mut GasMeter,
        data: Vec<u8>,
        salt: &[u8],
    ) -> Result<([u8; 32], ReturnData)> {
        self.ext.instantiates.push(InstantiateEntry {
            code_hash,
            endowment: 3, // endowment
            data: data.to_vec(),
            gas_left: gas_meter.gas_left,
            salt: salt.to_vec(),
        });

        self.invoke(code_hash, "deploy", data)
    }

    /// Call other contract
    pub fn call(
        &mut self,
        code_hash: [u8; 32],
        // value: u64,
        data: Vec<u8>,
    ) -> Result<ReturnData> {
        self.ext.transfers.push(TransferEntry {
            to: code_hash,
            value: 0,
            data: data.clone(),
        });

        self.invoke(code_hash, "call", data).map(|v| v.1)
    }
}
