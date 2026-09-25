use super::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Effect {
    Retrieve,
    Generate,
    Store(DevDeviceId),
}

#[derive(Clone, Default)]
struct Trace(Rc<RefCell<Vec<Effect>>>);

impl Trace {
    fn record(&self, effect: Effect) {
        self.0.borrow_mut().push(effect);
    }

    fn effects(&self) -> Vec<Effect> {
        self.0.borrow().clone()
    }
}

struct ScriptedStorage {
    retrievals: VecDeque<Result<Option<DevDeviceId>>>,
    store_result: Option<Result<()>>,
    trace: Trace,
}

impl ScriptedStorage {
    fn new(
        retrievals: Vec<Result<Option<DevDeviceId>>>,
        store_result: Result<()>,
        trace: Trace,
    ) -> Self {
        Self {
            retrievals: retrievals.into(),
            store_result: Some(store_result),
            trace,
        }
    }
}

impl Storage for ScriptedStorage {
    fn retrieve(&mut self) -> Result<Option<DevDeviceId>> {
        self.trace.record(Effect::Retrieve);
        self.retrievals
            .pop_front()
            .expect("unexpected storage retrieval")
    }

    fn store(&mut self, id: &DevDeviceId) -> Result<()> {
        self.trace.record(Effect::Store(id.clone()));
        self.store_result.take().expect("unexpected storage write")
    }
}

struct FixedIdGenerator {
    id: DevDeviceId,
    trace: Trace,
}

impl IdGenerator for FixedIdGenerator {
    fn generate(&mut self) -> DevDeviceId {
        self.trace.record(Effect::Generate);
        self.id.clone()
    }
}

fn fixed_id(value: u128) -> DevDeviceId {
    DevDeviceId(Uuid::from_u128(value))
}

fn generator(id: DevDeviceId, trace: &Trace) -> FixedIdGenerator {
    FixedIdGenerator {
        id,
        trace: trace.clone(),
    }
}

#[test]
fn test_expected_format() {
    let uuid = Uuid::new_v4();
    let id = DevDeviceId(uuid);
    let formatted = format!("{}", id);
    let mut buf = vec![0u8; uuid::fmt::Hyphenated::LENGTH];
    let hyphenated = uuid::fmt::Hyphenated::from_uuid(uuid);
    hyphenated.encode_lower(&mut buf);
    let expected = String::from_utf8(buf).expect("Failed to convert to String");
    assert_eq!(formatted, expected);
}

#[test]
fn initial_decision_returns_existing_id() {
    let id = fixed_id(1);
    assert_eq!(
        decide_initial(Some(id.clone())),
        InitialDecision::Return(id)
    );
}

#[test]
fn initial_decision_generates_when_id_is_missing() {
    assert_eq!(decide_initial(None), InitialDecision::Generate);
}

#[test]
fn post_store_decision_prefers_reloaded_id() {
    let generated = fixed_id(1);
    let reloaded = fixed_id(2);
    assert_eq!(
        decide_after_store(generated, Some(reloaded.clone())),
        reloaded
    );
}

#[test]
fn post_store_decision_falls_back_to_generated_id() {
    let generated = fixed_id(1);
    assert_eq!(decide_after_store(generated.clone(), None), generated);
}

#[test]
fn get_returns_none_for_empty_storage() {
    let trace = Trace::default();
    let mut storage = ScriptedStorage::new(vec![Ok(None)], Ok(()), trace.clone());

    assert_eq!(get_with_storage(&mut storage).unwrap(), None);
    assert_eq!(trace.effects(), vec![Effect::Retrieve]);
}

#[test]
fn get_or_generate_returns_existing_id_without_side_effects() {
    let existing = fixed_id(1);
    let trace = Trace::default();
    let mut storage = ScriptedStorage::new(vec![Ok(Some(existing.clone()))], Ok(()), trace.clone());
    let mut generator = generator(fixed_id(2), &trace);

    assert_eq!(
        get_or_generate_with(&mut storage, &mut generator).unwrap(),
        existing
    );
    assert_eq!(trace.effects(), vec![Effect::Retrieve]);
}

#[test]
fn get_or_generate_prefers_reloaded_id() {
    let generated = fixed_id(1);
    let reloaded = fixed_id(2);
    let trace = Trace::default();
    let mut storage = ScriptedStorage::new(
        vec![Ok(None), Ok(Some(reloaded.clone()))],
        Ok(()),
        trace.clone(),
    );
    let mut generator = generator(generated.clone(), &trace);

    assert_eq!(
        get_or_generate_with(&mut storage, &mut generator).unwrap(),
        reloaded
    );
    assert_eq!(
        trace.effects(),
        vec![
            Effect::Retrieve,
            Effect::Generate,
            Effect::Store(generated),
            Effect::Retrieve,
        ]
    );
}

#[test]
fn get_or_generate_falls_back_when_reloaded_id_is_missing() {
    let generated = fixed_id(1);
    let trace = Trace::default();
    let mut storage = ScriptedStorage::new(vec![Ok(None), Ok(None)], Ok(()), trace.clone());
    let mut generator = generator(generated.clone(), &trace);

    assert_eq!(
        get_or_generate_with(&mut storage, &mut generator).unwrap(),
        generated.clone()
    );
    assert_eq!(
        trace.effects(),
        vec![
            Effect::Retrieve,
            Effect::Generate,
            Effect::Store(generated),
            Effect::Retrieve,
        ]
    );
}

#[test]
fn get_or_generate_propagates_initial_retrieve_error() {
    let trace = Trace::default();
    let mut storage = ScriptedStorage::new(
        vec![Err(Error::StorageError("retrieve failed".to_string()))],
        Ok(()),
        trace.clone(),
    );
    let mut generator = generator(fixed_id(1), &trace);

    assert!(matches!(
        get_or_generate_with(&mut storage, &mut generator),
        Err(Error::StorageError(_))
    ));
    assert_eq!(trace.effects(), vec![Effect::Retrieve]);
}

#[test]
fn get_or_generate_propagates_second_retrieve_error() {
    let generated = fixed_id(1);
    let trace = Trace::default();
    let mut storage = ScriptedStorage::new(
        vec![
            Ok(None),
            Err(Error::StorageError("reload failed".to_string())),
        ],
        Ok(()),
        trace.clone(),
    );
    let mut generator = generator(generated.clone(), &trace);

    assert!(matches!(
        get_or_generate_with(&mut storage, &mut generator),
        Err(Error::StorageError(_))
    ));
    assert_eq!(
        trace.effects(),
        vec![
            Effect::Retrieve,
            Effect::Generate,
            Effect::Store(generated),
            Effect::Retrieve,
        ]
    );
}

#[test]
fn get_or_generate_propagates_already_set_race() {
    let generated = fixed_id(1);
    let trace = Trace::default();
    let mut storage = ScriptedStorage::new(vec![Ok(None)], Err(Error::AlreadySet), trace.clone());
    let mut generator = generator(generated.clone(), &trace);

    assert!(matches!(
        get_or_generate_with(&mut storage, &mut generator),
        Err(Error::AlreadySet)
    ));
    assert_eq!(
        trace.effects(),
        vec![Effect::Retrieve, Effect::Generate, Effect::Store(generated),]
    );
}
