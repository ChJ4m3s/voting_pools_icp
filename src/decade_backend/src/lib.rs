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
struct Poll {
    id: u64,
    title: String,
    description: String,
    votes: i128,
    created_at: u64,
    created_by: Principal,
    votes_vec: Vec<Principal>,
    category: u64,
    comments: Vec<Comment>,
    expiring_date: u64
}

#[derive(Deserialize, CandidType)]
struct SearchFilter {
    title: Option<String>,
    min_votes: Option<i128>,
    max_votes: Option<i128>,
    created_by: Option<Principal>,
    category: Option<u64>,
    min_created_at: Option<u64>,
    max_created_at: Option<u64>,
    expiring_before: Option<u64>,
    expiring_after: Option<u64>,
    description: Option<String>
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
struct SimpleInput {
    title: String,
    description: String,
}

#[derive(CandidType)]
struct CategoryResult {
    category: SimpleInput,
    id: u64
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
struct Comment {
    created_at: u64,
    comment: String,
    author: Principal
}

impl Storable for Poll {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Poll {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for SimpleInput {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}
impl BoundedStorable for SimpleInput {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER_POLLS: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static POLL_STORAGE: RefCell<StableBTreeMap<u64, Poll, Memory>> = RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
    ));

    static ID_COUNTER_CATEGORIES: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static CATEGORIES: RefCell<StableBTreeMap<u64, SimpleInput, Memory>> = RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
    ));
}

#[derive(candid::CandidType, Serialize, Deserialize)]
struct PollResult {
    id: u64,
    title: String,
    description: String,
    votes: i128,
    created_at: u64,
    created_by: Principal,
    expiring_date: u64
}

#[ic_cdk::query]
fn get_categories() -> Vec<CategoryResult> {
    CATEGORIES.with(|categories| {
        let categories_borrowed = categories.borrow();
        categories_borrowed.iter().map(|(id, category)| CategoryResult {
            id,
            category: category.clone()
        }).collect()
    })
}

#[ic_cdk::update]
fn add_category(category: SimpleInput) -> Option<CategoryResult> {
    let id = ID_COUNTER_CATEGORIES.with(|counter| {
        let current_value: u64 = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1)
    }).expect("cannot increment id counter");
    CATEGORIES.with(|service| service.borrow_mut().insert(id, category.clone()));
    Some(CategoryResult {
        id,
        category
    })
}

#[ic_cdk::query]
fn get_poll_result(id: u64) -> Result<PollResult, Error> {
    match _get_poll(&id) {
        Some(poll) => {
            let poll_result = PollResult {
                id: poll.id,
                title: poll.title,
                votes: poll.votes,
                description: poll.description,
                created_at: poll.created_at,
                created_by: poll.created_by,
                expiring_date: poll.expiring_date
            };
            Ok(poll_result)
        },
        None => Err(Error::NotFound {
            msg: format!("a poll with id={} not found", id),
        }),
    }
}

fn _get_poll(id: &u64) -> Option<Poll> {
    POLL_STORAGE.with(|s| s.borrow().get(id))
}

#[ic_cdk::query]
fn get_category(id: u64) -> Result<CategoryResult, Error> {
    CATEGORIES.with(|s| {
        let category = s.borrow().get(&id);
        match category {
            Some(c) => {
                Ok(CategoryResult {
                    category: c,
                    id
                })
            }
            None => {
                Err(Error::NotFound {
                    msg: format!(
                    "could't find a category with id={}.", id
                    ),
                })
            }
        }

    })
}

#[ic_cdk::query]
fn get_polls(filters: SearchFilter) -> Vec<Poll> {
    POLL_STORAGE.with(|s| s.borrow().iter().filter(|(_, poll)| {
        let mut res = true;
        match &filters.title {
            Some(t) => {
                let lowercase_filter = t.to_lowercase();
                let separated_words: Vec<String> = lowercase_filter.split_whitespace().map(|w| w.to_string()).collect();
                let lowercase_title = poll.title.to_lowercase();
                res = separated_words.iter().any(|word| lowercase_title.contains(word));
            }
            None => {}
        }
        if res {
            match filters.min_votes {
                Some(n) => res = poll.votes >= n,
                None => {},
            }
        }
        if res {
            match filters.max_votes {
                Some(n) => res = poll.votes <= n,
                None => {},
            }
        }
        if res {
            match filters.created_by {
                Some(creator) => res = poll.created_by == creator,
                None => {}
            }
        }
        if res {
            match filters.category {
                Some(category) => res = poll.category == category,
                None => {}
            }
        }
        if res {
            match filters.min_created_at {
                Some(d) => res = poll.created_at >= d,
                None => {}
            }
        }
        if res {
            match filters.max_created_at {
                Some(d) => res = poll.created_at <= d,
                None => {}
            }
        }
        if res {
            match filters.expiring_after {
                Some(d) => res = poll.expiring_date > d,
                None => {}
            }
        }
        if res {
            match filters.expiring_before {
                Some(d) => res = poll.expiring_date < d,
                None => {}
            }
        }
        if res {
            match &filters.description {
                Some(d) => {
                    let lowercase_filter = d.to_lowercase();
                    let separated_words: Vec<String> = lowercase_filter.split_whitespace().map(|w| w.to_string()).collect();
                    let lowercase_description = poll.description.to_lowercase();
                    res = separated_words.iter().any(|word| lowercase_description.contains(word));
                }
                None => {}
            }
        }
        res
    })
        .map(|(_id, poll)| poll.clone())
        .collect()
    )
}

#[ic_cdk::update]
fn create_poll(poll: SimpleInput, category: u64, validity_period: u64) -> Result<Poll, Error> {
    let id = ID_COUNTER_POLLS.with(|counter| {
        let current_value: u64 = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1)
    }).expect("cannot increment id counter");
    CATEGORIES.with(|s| {
        let category_res = s.borrow().get(&category);
        match category_res {
            Some(_) => {
                let poll = Poll {
                    id,
                    title: poll.title,
                    description: poll.description,
                    created_at: time(),
                    votes: 0,
                    votes_vec: vec![],
                    created_by:  ic_cdk::caller(),
                    category,
                    comments: vec![],
                    expiring_date: time() + validity_period
                };
                do_insert(&poll);
                Ok(poll)
            }
            None => Err(Error::NotFound {
                msg: format!(
                "could't find a category with id={}.", category
                ),
            })
        }
    })
}

fn do_insert(message: &Poll) {
    POLL_STORAGE.with(|service| service.borrow_mut().insert(message.id, message.clone()));
}

#[ic_cdk::update]
fn vote_poll(poll_id: u64, vote: i8) -> Result<PollResult, Error> {
    match POLL_STORAGE.with(|service| service.borrow().get(&poll_id)) {
        Some(mut poll) => {
            if vote > 1 || vote < -1 {
                Err(Error::NotFound {
                    msg: format!(
                        "value {} is not valid", vote
                    ),
                })
            } else {
                if time() < poll.expiring_date {
                    let already_voted = poll.votes_vec.iter().any(|voter| *voter == ic_cdk::caller());

                    if !already_voted {
                        poll.votes += vote as i128;
                        poll.votes_vec.push(ic_cdk::caller());
                        do_insert(&poll);
                        let poll_result = PollResult {
                            id: poll_id,
                            title: poll.title,
                            description: poll.description,
                            votes: poll.votes,
                            created_at: poll.created_at,
                            created_by: poll.created_by,
                            expiring_date: poll.expiring_date
                        };
                        Ok(poll_result)
                    } else {
                        Err(Error::AlreadyVoted {
                            msg: format!(
                                "voter with principal {} already voted for poll with id={}", ic_cdk::caller(), poll_id
                            ),
                        })
                    }
                } else {
                    Err(Error::AlreadyVoted {
                        msg: "poll expired".parse().unwrap(),
                    })
                }
            }
        },
        None => Err(Error::NotFound {
            msg: format!(
                "could't update a poll with id={}.", poll_id
            ),
        }),
    }
}

#[ic_cdk::update]
fn add_comment(poll_id: u64, comment: String) -> Result<PollResult, Error> {
    match POLL_STORAGE.with(|service| service.borrow().get(&poll_id)) {
        Some (mut poll) => {
            let comment_obj = Comment{
                author: ic_cdk::caller(),
                created_at: time(),
                comment
            };
            poll.comments.push(comment_obj);
            do_insert(&poll);
            let poll_result = PollResult {
                id: poll_id,
                title: poll.title,
                description: poll.description,
                votes: poll.votes,
                created_at: poll.created_at,
                created_by: poll.created_by,
                expiring_date: poll.expiring_date
            };
            Ok(poll_result)
        },
        None => Err(Error::NotFound {
            msg: format!(
                "could't update a poll with id={}.", poll_id
            ),
        }),
    }
}

#[ic_cdk::query]
fn get_comments(poll_id: u64) -> Result<Vec<Comment>, Error> {
    match POLL_STORAGE.with(|service| service.borrow().get(&poll_id)) {
        Some (poll) => {
            Ok(poll.comments)
        },
        None => Err(Error::NotFound {
            msg: format!(
                "could't find a poll with id={}.", poll_id
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