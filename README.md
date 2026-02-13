# Stahlwerk Extension – Stahlwerk Integration Extension for control

Note: Requires the beas-bsl integration module


---

## FF01 Module

Provides the integration layer for the FF01 machine.

This module acts as an intermediary between the FF01 machine and the server.


---

### Entry

Struct that holds the necessary data for the machine to process it's task

### ProxyClient

Proxy for a beas_bsl::Client that exposes following requests

#### get_next_entry
Retrieves the next entry the machine should process.

#### get_scrap_quantity
Returns the current scrap_quantity for a given workorder position

#### finalize
Finalizes the workorder step by closing the WorkorderRouting 10/10 and submitting a backflish POST request

### FF01 Pipeline

1. App polls for a new entry via `ClientProxy::get_next_entry`
2. `User` submits a new workorder
3. App receives new `Entry` from `ClientProxy::get_next_entry`
4. App notifies machine to start counting/measuring and passes bounds from `Entry`
5. App polls `ClientProxy::get_scrap_quantity` to detect a change in scrap_quantity.
6. `User` submits scrap quantity
7. App detects change in scrap_quantity and notifies machine to stop counting/measuring
8. App finalizes data and submits the data to via `ClientProxy::finalize`
9. Goto 1