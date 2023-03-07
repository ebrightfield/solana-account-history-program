## Account History Program

This program does one thing and does it well.
It records historical Solana on-chain account state, on-chain on Solana.

With this program, you can:
- Create oracle price history feeds, from which common trading statistics can be derived.
- Store an on-chain record of token balance changes.
- Create on-chain logic that gates certain operations by historical presence of certain on-chain data.
- Gate your program's mutation with update logic, so that the last N state updates are always visible on-chain and easily available.

### How It Works
You create an `AccountHistory` account, which is configured with:
- Some target account whose state is to be copied.
- Some set of data regions to copy.
- A capacity for some number of snapshots.

In either permissionless or permissioned fashion, an `AccountHistory` account can
be updated with the latest snapshot of the target data on the target account,
labeled with the slot number at the time of the data snapshot.

### Other Details
- The slot number is prepended to every element of historical data, so the element size will always be >8 bytes long.
- The program knows nothing other than that it is copying blobs of bytes. It is up to the administrator
to configure their `AccountHistory` to copy the correct bytes. And it is up to clients to choose how
to interpret those bytes. There are helper functions in the SDK to assist in client code.
- Accounts can have a minimum slot delay between updates. This enforces a "maximum resolution" to the historical
data feed, and a minimum age of the oldest element which is equal to `C * D`, where `C` is the capacity and `D` is the minimum slot delay.
- Accounts can be closed, and their lamports retrieved.
- Admins can add a minimum delay to closing an `AccountHistory`. This allows administrators to relinquish the power of surprise over on-chain consumers of `AccountHistory` data.