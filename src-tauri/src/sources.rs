//! Veritabanı destekli değer kaynağı: planlardaki `fromProfile` / `fromSnapshot` işlemlerinin
//! değerlerini, değerler ön uca hiç gönderilmeden çözer.

use crate::db::Db;
use crate::env::model::name_key;
use crate::env::plan::ValueSource;
use crate::env::{Scope, ValueKind};
use crate::error::{AppError, AppResult};
use crate::profiles::{model::Profile, model::VarAction, store as profiles};
use crate::snapshots::store::{self as snapshots, SnapshotItem};
use std::cell::RefCell;
use std::collections::HashMap;

pub struct DbSources<'a> {
    db: &'a Db,
    imports: Option<&'a crate::io::import::ImportCache>,
    profiles: RefCell<HashMap<i64, Profile>>,
    snapshots: RefCell<HashMap<i64, Vec<SnapshotItem>>>,
}

impl<'a> DbSources<'a> {
    pub fn new(db: &'a Db) -> Self {
        DbSources {
            db,
            imports: None,
            profiles: RefCell::default(),
            snapshots: RefCell::default(),
        }
    }
}

impl<'a> DbSources<'a> {
    pub fn with_imports(mut self, cache: &'a crate::io::import::ImportCache) -> Self {
        self.imports = Some(cache);
        self
    }
}

impl ValueSource for DbSources<'_> {
    fn import_var(&self, token: &str, name: &str) -> AppResult<(String, Option<ValueKind>)> {
        let cache = self
            .imports
            .ok_or_else(|| AppError::NotFound(name.to_string()))?;
        crate::io::import::import_value(&cache.get(token)?, name)
    }

    fn profile_var(&self, profile_id: i64, name: &str) -> AppResult<(String, Option<ValueKind>)> {
        if !self.profiles.borrow().contains_key(&profile_id) {
            let p = profiles::load_full(self.db, profile_id)?;
            self.profiles.borrow_mut().insert(profile_id, p);
        }
        let cache = self.profiles.borrow();
        let v = cache[&profile_id]
            .vars
            .iter()
            .find(|v| name_key(&v.name) == name_key(name) && v.action == VarAction::Set)
            .ok_or_else(|| AppError::NotFound(name.to_string()))?;
        let value = v
            .value
            .clone()
            .ok_or_else(|| AppError::NotFound(name.to_string()))?;
        Ok((value, v.kind))
    }

    fn snapshot_item(
        &self,
        snapshot_id: i64,
        scope: Scope,
        name: &str,
    ) -> AppResult<Option<(String, ValueKind)>> {
        if !self.snapshots.borrow().contains_key(&snapshot_id) {
            let items = snapshots::items(self.db, snapshot_id)?;
            self.snapshots.borrow_mut().insert(snapshot_id, items);
        }
        let cache = self.snapshots.borrow();
        let it = cache[&snapshot_id]
            .iter()
            .find(|i| i.scope == scope && name_key(&i.name) == name_key(name))
            .ok_or_else(|| AppError::NotFound(name.to_string()))?;
        Ok(match (it.present, &it.value) {
            (true, Some(v)) => Some((v.clone(), it.kind)),
            _ => None,
        })
    }
}
