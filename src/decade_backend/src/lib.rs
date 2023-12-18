#[macro_use]
extern crate serde;
use candid::{Decode, Encode, Principal, CandidType};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Defining struct types and storable

#[derive(CandidType, Clone, Serialize, Deserialize)]
struct Pool {
    id: u64,
    title: String,
    description: String,
    votes: i128,
    created_at: u64,
    created_by: Principal,
    votes_vec: Vec<Principal>
}


#[derive(CandidType, Clone, Serialize, Deserialize)]
struct PoolInput<'a> {
    title: Cow<'a, str>,
    description: Cow<'a, str>,
}

impl Storable for Pool {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Pool {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static POOL_STORAGE: RefCell<StableBTreeMap<u64, Pool, Memory>> = RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

#[derive(candid::CandidType, Serialize, Deserialize)]
struct PoolResult {
    id: u64,
    title: String,
    description: String,
    votes: i128,
    created_at: u64,
    created_by: Principal,
}

#[ic_cdk::query]
fn get_pool_result(id: u64) -> Result<PoolResult, Error> {
    match _get_pool(&id) {
        Some(pool) => {
            let pool_result = PoolResult {
                id: pool.id,
                title: pool.title,
                votes: pool.votes,
                description: pool.description,
                created_at: pool.created_at,
                created_by: pool.created_by
            };
            Ok(pool_result)
        },
        None => Err(Error::NotFound {
            msg: format!("a pool with id={} not found", id),
        }),
    }
}

fn _get_pool(id: &u64) -> Option<Pool> {
    POOL_STORAGE.with(|s| s.borrow().get(id))
}

#[ic_cdk::update]
fn create_pool(pool: PoolInput) -> Option<Pool> {
    let id = ID_COUNTER.with(|counter| {
        let current_value: u64 = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1)
    }).expect("cannot increment id counter");
    let pool = Pool {
        id,
        title: pool.title,
        description: pool.description,
        created_at: time(),
        votes: 0,
        votes_vec: vec![],
        created_by:  ic_cdk::caller(),
    };
    do_insert(&pool);
    Some(pool)
}

fn do_insert(message: &Pool) -> Result<(), Error> {
    POOL_STORAGE.with(|service| {
        service.borrow_mut().insert(message.id, message);
        Ok(())
    })
}

#[ic_cdk::update]
fn vote_pool(pool_id: u64, vote: i8) -> Result<PoolResult, Error> {
    match POOL_STORAGE.with(|service| service.borrow().get(&pool_id)) {
        Some(mut pool) => {
            if vote > 1 || vote < -1 {
                Err(Error::NotFound {
                    msg: format!(
                        "value {} is not valid", vote
                    ),
                })
            } else {
                let already_voted = pool.votes_vec.iter().any(|voter| *voter == ic_cdk::caller());

                if !already_voted {
                    pool.votes += vote as i128;
                    pool.votes_vec.push(ic_cdk::caller());
                    do_insert(&pool);
                    let pool_result = PoolResult {
                        id: pool_id,
                        title: pool.title,
                        description: pool.description,
                        votes: pool.votes,
                        created_at: pool.created_at,
                        created_by: pool.created_by
                    };
                    Ok(pool_result)
                } else {
                    Err(Error::AlreadyVoted {
                        msg: format!(
                            "voter with principal {} already voted for pool with id={}", ic_cdk::caller(), pool_id
                        ),
                    })
                }
            }
        },
        None => Err(Error::NotFound {
            msg: format!(
                "could't update a pool with id={}. message not found", pool_id
            ),
        }),
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    AlreadyVoted {msg: String},
    NotValidVote {msg: String},
}

ic_cdk::export_candid!();