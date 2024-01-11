# Voting system

This is a Rust implementation for a voting poll. The canister will allow to add proposals and vote for approval or rejection.

## Technoloties used

- **Rust**: backend implementation
- **Candid**: canister interface
- **Internet computer (IC)**: librarrequired libraries
- **ICP principals**: identify users

## Functionalities

- **create_poll**: Create a new poll with title and description;
- **get_poll_result**: Get the result for a poll and its details;
- **vote_poll**: If user has not already voted, can vote by typing 1 for approval, 0 for neutral, -1 for rejection (neutral is considered as a vote);
- **add_categories**: Allow polls to be categorized (e.g., technology, politics, entertainment). This helps users find polls that interest them more easily;
- **add_comment**: Allow users to add comments to existing polls;
- **get_categories**: Gets the full list of existing categories;
- **get_category**: Gets the details of selected category;
- **get_comments**: Gets the comments added to a poll;
- **get_polls**: Filter polls by (title, description, min and max votes, min and max creation timestamp, creator, category, and expiring timestamp) and get the resulting list.

## How to run locally

- `dfx start --clean`
- `./did.sh`
- `dfx deploy`

Note: if `./did.sh` is not working, add `+x` permission.