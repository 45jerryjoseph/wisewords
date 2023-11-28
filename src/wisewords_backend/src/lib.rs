#[macro_use]
extern crate serde;
use candid::{Decode, Encode, Principal};
use ic_cdk::api::{caller, time};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::cmp::Reverse;
use std::{borrow::Cow, cell::RefCell};
use validator::Validate;

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct Contributor {
    id: u64,
    user_principal: Principal,
    username: String,
    email: String,
    age: u32,
    created_at: u64,
    updated_at: Option<u64>,
}

// Implement Storable and BoundedStorable traits for Contributor
impl Storable for Contributor {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Contributor {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Quote {
    id: u64,
    contributor_id: u64,
    author: String,
    text: String,
    category: String,
    created_at: u64,
    updated_at: Option<u64>,
}

// Implement Storable and BoundedStorable traits for Quote
impl Storable for Quote {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Quote {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static CONTRIBUTOR_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );
    static QUOTE_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))), 0)
            .expect("Cannot create a counter")
    );
    static CONTRIBUTOR_STORAGE: RefCell<StableBTreeMap<u64, Contributor, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static QUOTE_STORAGE: RefCell<StableBTreeMap<u64, Quote, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

}

#[derive(candid::CandidType, Serialize, Deserialize, Default, Validate)]
struct ContributorPayload {
    #[validate(length(min = 3))]
    username: String,
    email: String,
    #[validate(range(min = 18))]
    age: u32,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default, Validate)]
struct QuotePayload {
    contributor_id: u64,
    author: String,
    #[validate(length(min = 1))]
    text: String,
    #[validate(length(min = 3))]
    category: String,
}

// Contributer CRUD

// get all contributors
#[ic_cdk::query]
fn get_all_contributors() -> Result<Vec<Contributor>, Error> {
    let contributors_map: Vec<(u64, Contributor)> =
        CONTRIBUTOR_STORAGE.with(|service| service.borrow().iter().collect());
    let contributors: Vec<Contributor> = contributors_map
        .into_iter()
        .map(|(_, contributor)| contributor)
        .collect();

    if !contributors.is_empty() {
        Ok(contributors)
    } else {
        Err(Error::NotFound {
            msg: "No contributors found.".to_string(),
        })
    }
}

#[ic_cdk::query]
fn get_contributor(id: u64) -> Result<Contributor, Error> {
    match _get_contributor(&id) {
        Some(contributor) => Ok(contributor),
        None => Err(Error::NotFound {
            msg: format!("Searched but Contributor with id={} not found", id),
        }),
    }
}
// a helper method to get a contributor by id.
fn _get_contributor(id: &u64) -> Option<Contributor> {
    CONTRIBUTOR_STORAGE.with(|service| service.borrow().get(id))
}

#[ic_cdk::update]
fn add_contributor(contrib: ContributorPayload) -> Result<Contributor, Error> {
    let check_payload = contrib.validate();
    if check_payload.is_err() {
        return Err(Error::ValidationFailed {
            content: check_payload.err().unwrap().to_string(),
        });
    }
    let id = CONTRIBUTOR_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let contributor = Contributor {
        id,
        user_principal: caller(),
        username: contrib.username,
        email: contrib.email,
        age: contrib.age,
        created_at: time(),
        updated_at: None,
    };
    do_insert_contributor(&contributor);
    Ok(contributor)
}

#[ic_cdk::update]
fn update_contributor(id: u64, payload: ContributorPayload) -> Result<Contributor, Error> {
    let check_payload = payload.validate();
    if check_payload.is_err() {
        return Err(Error::ValidationFailed {
            content: check_payload.err().unwrap().to_string(),
        });
    }
    match CONTRIBUTOR_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut contributor) => {
            // Validates whether caller is the owner
            let check_if_owner = _check_if_owner(&contributor);
            if check_if_owner.is_err() {
                return Err(check_if_owner.err().unwrap());
            }
            contributor.username = payload.username;
            contributor.email = payload.email;
            contributor.age = payload.age;
            contributor.updated_at = Some(time());
            do_insert_contributor(&contributor);
            Ok(contributor)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update a Contributor  with id={}. Contributor not found",
                id
            ),
        }),
    }
}

// helper method to perform insert.
fn do_insert_contributor(contributor: &Contributor) {
    CONTRIBUTOR_STORAGE.with(|service| {
        service
            .borrow_mut()
            .insert(contributor.id, contributor.clone())
    });
}

#[ic_cdk::update]
fn delete_contributor(id: u64) -> Result<Contributor, Error> {
    let contributor =
        _get_contributor(&id).expect(&format!("couldn't find a contributor with id={}", id));
    // Validates whether caller is the owner
    let check_if_owner = _check_if_owner(&contributor);
    if check_if_owner.is_err() {
        return Err(check_if_owner.err().unwrap());
    }
    match CONTRIBUTOR_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(contributor) => Ok(contributor),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a message with id={}. message not found.",
                id
            ),
        }),
    }
}

// Qoutes

#[ic_cdk::query]
fn get_all_quotes() -> Result<Vec<Quote>, Error> {
    let quotes_map: Vec<(u64, Quote)> =
        QUOTE_STORAGE.with(|service| service.borrow().iter().collect());
    let quotes: Vec<Quote> = quotes_map.into_iter().map(|(_, quote)| quote).collect();

    if !quotes.is_empty() {
        Ok(quotes)
    } else {
        Err(Error::NotFound {
            msg: "No quotes found.".to_string(),
        })
    }
}

#[ic_cdk::query]
fn get_quote(id: u64) -> Result<Quote, Error> {
    match _get_quote(&id) {
        Some(quote) => Ok(quote),
        None => Err(Error::NotFound {
            msg: format!("Searched but Quote with id={} not found", id),
        }),
    }
}
// a helper method to get a contributor by id.
fn _get_quote(id: &u64) -> Option<Quote> {
    QUOTE_STORAGE.with(|service| service.borrow().get(id))
}

// get recent quotes that were created eg: the last five

#[ic_cdk::query]
fn get_recent_quotes() -> Result<Vec<Quote>, Error> {
    let quotes_map: Vec<(u64, Quote)> =
        QUOTE_STORAGE.with(|service| service.borrow().iter().collect());

    // Sort the quotes by created_at timestamp in reverse order to get the most recent ones first
    let mut sorted_quotes: Vec<Quote> = quotes_map.into_iter().map(|(_, quote)| quote).collect();
    sorted_quotes.sort_by_key(|quote| Reverse(quote.created_at));

    // Take the first 5 quotes (the most recent ones)
    let recent_quotes: Vec<Quote> = sorted_quotes.into_iter().take(5).collect();

    if !recent_quotes.is_empty() {
        Ok(recent_quotes)
    } else {
        Err(Error::NotFound {
            msg: "No recent quotes found.".to_string(),
        })
    }
}

// Function to get quotes by a specified category

#[ic_cdk::query]
fn get_quotes_by_category(category: String) -> Result<Vec<Quote>, Error> {
    let quotes_map: Vec<(u64, Quote)> =
        QUOTE_STORAGE.with(|service| service.borrow().iter().collect());

    // Filter quotes by the provided category (case insensitive)
    let quotes_in_category: Vec<Quote> = quotes_map
        .into_iter()
        .map(|(_, quote)| quote)
        .filter(|quote| quote.category.to_lowercase() == category.to_lowercase())
        .collect();

    if !quotes_in_category.is_empty() {
        Ok(quotes_in_category)
    } else {
        Err(Error::NotFound {
            msg: format!("No quotes found in category: {}", category),
        })
    }
}

#[ic_cdk::update]

fn add_quote(quotepayload: QuotePayload) -> Result<Quote, Error> {
    let check_payload = quotepayload.validate();
    if check_payload.is_err() {
        return Err(Error::ValidationFailed {
            content: check_payload.err().unwrap().to_string(),
        });
    }
    let contributor = _get_contributor(&quotepayload.contributor_id).expect(&format!(
        "couldn't find a contributor with id={}",
        quotepayload.contributor_id
    ));
    let check_if_owner = _check_if_owner(&contributor);
    if check_if_owner.is_err() {
        return Err(check_if_owner.err().unwrap());
    }
    let id = QUOTE_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let quote = Quote {
        id,
        contributor_id: quotepayload.contributor_id,
        author: quotepayload.author,
        text: quotepayload.text,
        category: quotepayload.category,
        created_at: time(),
        updated_at: None,
    };
    do_insert_quote(&quote);
    Ok(quote)
}

#[ic_cdk::update]
fn update_quote(id: u64, payload: QuotePayload) -> Result<Quote, Error> {
    let check_payload = payload.validate();
    if check_payload.is_err() {
        return Err(Error::ValidationFailed {
            content: check_payload.err().unwrap().to_string(),
        });
    }
    match QUOTE_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut quote) => {
            let contributor = _get_contributor(&quote.contributor_id).expect(&format!(
                "couldn't find a contributor with id={}",
                quote.contributor_id
            ));
            let check_if_owner = _check_if_owner(&contributor);
            if check_if_owner.is_err() {
                return Err(check_if_owner.err().unwrap());
            }
            // ensures that the new contributor_id is an id that is valid
            assert!(
                _get_contributor(&payload.contributor_id).is_some(),
                "Invalid new contributor id"
            );
            quote.contributor_id = payload.contributor_id;
            quote.author = payload.author;
            quote.text = payload.text;
            quote.category = payload.category;
            quote.updated_at = Some(time());
            do_insert_quote(&quote);
            Ok(quote)
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update a Quote with id={}. Quote not found", id),
        }),
    }
}

// helper method to perform insert.
fn do_insert_quote(quote: &Quote) {
    QUOTE_STORAGE.with(|service| service.borrow_mut().insert(quote.id, quote.clone()));
}

// Helper function to check whether the caller is the owner of the contributor profile
fn _check_if_owner(contributor: &Contributor) -> Result<(), Error> {
    if contributor.user_principal.to_string() != caller().to_string() {
        return Err(Error::AuthenticationFailed {
            msg: format!(
                "Caller={} isn't the owner of the contributor with id={}",
                caller(),
                contributor.id
            ),
        });
    } else {
        Ok(())
    }
}

#[ic_cdk::update]
fn delete_quote(id: u64) -> Result<Quote, Error> {
    let contributor =
        _get_contributor(&id).expect(&format!("couldn't find a contributor with id={}", id));
    let check_if_owner = _check_if_owner(&contributor);
    if check_if_owner.is_err() {
        return Err(check_if_owner.err().unwrap());
    }
    match QUOTE_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(quote) => Ok(quote),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a message with id={}. message not found.",
                id
            ),
        }),
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    ValidationFailed { content: String },
    AuthenticationFailed { msg: String },
}

// need this to generate candid
ic_cdk::export_candid!();
