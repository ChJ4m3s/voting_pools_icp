# Voting system

This is a Rust implementation for a voting pool. The canister will allow to add proposals and vote for approval or rejection.

## Technoloties used

- **Rust**: backend implementation
- **Candid**: canister interface
- **Internet computer (IC)**: librarrequired libraries
- **ICP principals**: identify users

## Functionalities

- **create_pool**: Create a new pool with title and description;
- **get_pool_result**: Get the result for a pool and its details;
- **vote_pool**: If user has not already voted, can vote by typing 1 for approval, 0 for neutral, -1 for rejection (neutral is considered as a vote).

## How to run locally

- `dfx start --clean`
- `./did.sh`
- `dfx deploy`

Note: if `./did.sh` is not working, add `+x` permission.