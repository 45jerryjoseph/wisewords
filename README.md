# Wisewords

**Problem Statement**:

For an Aim to  seek to manage a repository of quotes contributed by various individuals. **Wisewords** system aims to facilitate the collection, organization, and retrieval of these quotes and contributor details.

The system aims to inspire and create awareness as well as a smile to  those viewing by even starting the day with a Quote 

The Contributor explores to come up with more Quotes, and bring to lime light the quotes which Can be placed in Categories such as :
 - **Funny  quotes**  
 - **friendship quotes** 
 - **Motivational quotes**
 - **leadership quotes**


To get started, you might want to explore the project directory structure and the default configuration file. Working with this project in your development environment will not affect any production deployment or identity tokens.

## Overview
The code includes structures and methods to manage contributors and quotes within a system. It utilizes the Internet Computer's APIs and libraries for memory management, CRUD operations, and error handling.

External Dependencies
  
  - **Serde**: External crate for serialization and deserialization.
  - **Candid**: Used for encoding and decoding types.
  - **ic_cdk**: Provides access to Internet Computer APIs.
  - **ic_stable_structures**: Includes memory management functionalities.
  - **std**: Standard Rust library components.

## Structures

`Contributor`

- Represents a contributor with the fields for `id`, `username`, `email`, `age`, `created_at` and `updated_at`

`Quote`

- Represents a quote with fields for `id`,`contributor_id`,`author`, `text`, `category`, `created_at` and `updated_at`


## Storable Traits

Both `Contributor `and `Quote` implement the `Storable` and `BoundedStorable `traits to manage serialization and storage size limits.

## Local Variables 

  - **Memory Manager** : Handles memory management.
  - **QUOTE_ID_COUNTER**: Manages unique IDs for  quotes.
  - **CONTRIBUTOR_ID_COUNTER**: Manages unique IDs for contributors.
  - **CONTRIBUTOR_STORAGE**: Stores contributors.
  - **QUOTE_STORAGE**: Stores quotes.

## Payload Structures
  - ContributorPayload: Struct to hold contributor information for CRUD operations.
  - QuotePayload: Struct to hold quote information for CRUD operations.

## Contributor Operations

  - `get_all_contributors()`: Retrieves all contributors.
  - `get_contributor(id: u64)`: Retrieves a specific contributor by ID.
  - `add_contributor(contrib: ContributorPayload)`: Adds a new contributor.
  - `update_contributor(id: u64, payload: ContributorPayload)`:   - Updates contributor information.
  - `delete_contributor(id: u64)`: Deletes a contributor by ID.

## Quote Operations

  - `get_all_quotes()`: Retrieves all quotes.
  - `get_quote(id: u64)`: Retrieves a specific quote by ID.
  - `get_recent_quotes()`: Retrieves the most recent quotes.
  - `get_quotes_by_category(category: String)`: Retrieves quotes by a specified category.
  - `add_quote(quotepayload: QuotePayload)`: Adds a new quote.
  - `update_quote(id: u64, payload: QuotePayload)`: Updates quote information.
  - `delete_quote(id: u64):` Deletes a quote by ID.


## Error Handling 

  - Defines an `Error` enum to handle 'Not Found' cases for cleaner error messages.

## Candid Inergration 
  -   Exports the code to generate Candid interface.


To learn more before you start working with Wisewords, see the following documentation available online:

- [Quick Start](https://internetcomputer.org/docs/current/developer-docs/setup/deploy-locally)
- [SDK Developer Tools](https://internetcomputer.org/docs/current/developer-docs/setup/install)
- [Rust Canister Development Guide](https://internetcomputer.org/docs/current/developer-docs/backend/rust/)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/current/developer-docs/backend/candid/)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd Wisewords/
dfx help
dfx canister --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```bash
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.

If you are making frontend changes, you can start a development server with

```bash
npm start
```

Which will start a server at `http://localhost:8080`, proxying API requests to the replica at port 4943.

### Note on frontend environment variables

If you are hosting frontend code somewhere without using DFX, you may need to make one of the following adjustments to ensure your project does not fetch the root key in production:

- set`DFX_NETWORK` to `ic` if you are using Webpack
- use your own preferred method to replace `process.env.DFX_NETWORK` in the autogenerated declarations
  - Setting `canisters -> {asset_canister_id} -> declarations -> env_override to a string` in `dfx.json` will replace `process.env.DFX_NETWORK` with the string in the autogenerated declarations
- Write your own `createActor` constructor
