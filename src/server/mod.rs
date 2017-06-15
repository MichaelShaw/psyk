


// we need to rethink this in terms of:
// 1. State Cells. (Client, Game)
// 2. State Transitions to cells. (and how we determined if these are allowed ... rejecting of state transitions)
// 3. Subscribers to state cells.
// 4. Recordings of state transitions are replays.


// 1. match making server
// - Real time loop (ticks + sleeps)
// - Event driven loop (for turn based games)

// Unsure quite how we integrate AI with this. AI events aren't .... simple 
// .... unless they're basically injected player input
