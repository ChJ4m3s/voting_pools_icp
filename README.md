# Voting system

This is a Rust implementation for a voting poll. The canister will allow to add proposals and vote for approval or rejection.

## Technoloties used

- **Rust**: backend implementation
- **Candid**: canister interface
- **Internet computer (IC)**: librarrequired libraries
- **ICP principals**: identify users

## Functionalities

- **create_poll(poll: {title: text, description: text}, category: u64, validity_period: u64)**: Create a new poll with title and description. The category field asks for the id of an existing category; if the category id does not exist, the function returns an error. The validity period is the timestamp of the expirity date, after which no more votes can be added.
- **get_poll_result(id: u64)**: Get the result for a poll given its it and its details. If the pool id does not exist, the function returns an error.
- **vote_poll(poll_id: u64, vote: i8)**: If user has not already voted, can vote by typing 1 for approval, 0 for neutral, -1 for rejection (neutral is considered as a vote). It returns an error if the poll does not exist, the vote is not valid (different from -1, 0, or 1), or the poll expired.
- **add_category(category: {name: text, description: text})**: Allow polls to be categorized (e.g., technology, politics, entertainment). This helps users find polls that interest them more easily. It takes in input the name and description of the category and returns its id.
- **add_comment(poll_id: u64, comment: text)**: Allow users to add comments to existing polls. If the poll does not exist, the function returns an error.
- **get_categories()**: Gets the full list of existing categories.
- **get_category(id: u64)**: Gets the details of selected category. If the category does not exist, an error is returned.
- **get_comments(poll_id: u64)**: Gets the comments added to a poll. If the poll does not exist, an error is returned.
- **get_polls(filters: {title : opt text, max_votes : opt int, min_votes : opt int, description : opt text, max_created_at : opt nat64, created_by : opt principal, expiring_before : opt nat64, category : opt nat64, expiring_after : opt nat64, min_created_at : opt nat64})**: Filter polls by (title, description, min and max votes, min and max creation timestamp, creator, category, and expiring timestamp) and get the resulting list.

## How to run locally

- `dfx start --clean`
- `./did.sh`
- `dfx deploy`

Note: if `./did.sh` is not working, add `+x` permission.
