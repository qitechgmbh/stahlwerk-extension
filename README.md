# Stahlwerk Extension – Stahlwerk Integration Extension for control

Note: Requires the `beas-bsl` integration


---

## Modules


---

### FF01 - Description

Provides the integration layer for the FF01 machine.

This module acts as an intermediary between the FF01 machine and the server.


---

### FF01 - Datatypes

#### Entry

Struct that holds all necessary data for the machine to process its task.


---

#### Bounds

Defines the minimum, maximum, and desired values for a given parameter.


---

#### ProxyClient

A proxy wrapper around `beas_bsl::Client` that provides a simplified interface for machine-related operations. It exposes the following requests:

- **get_next_entry**  
  Retrieves the next entry that the machine is scheduled to process.

- **get_scrap_quantity**  
  Returns the current `scrap_quantity` for a specific work order position.

- **finalize**  
  Completes the current work order step by closing `WorkorderRouting (10/10)` and submitting the corresponding backflush `POST` request to the server.


---

#### ResponseError

Errors returned by the underlying proxy client handler/worker:

- **Client**  
  Error received from the `beas_bsl::Client`.

- **InvalidData**  
  The client succeeded, but the received data does not meet the expected requirements.


---

#### TransactionError

Errors returned by any of `ProxyClient`'s requests:

- **Pending**  
  Request is still pending and should be polled again. **Action:** Retry.

- **ChannelFull**  
  Failed to submit the request because the channel is full. **Action:** Retry.

- **ChannelClosed**  
  The underlying worker terminated; no further requests can be processed. **Action:** Create a new proxy or abort.

- **TagMismatch**  
  User attempted to poll a different request type than the pending one. **Action:** Only poll the submitted request until a response is received.

- **Response**  
  Received an error response from the worker. **Action:** Handle accordingly.


---

### FF01 - Pipeline

1. App polls for a new entry via `ClientProxy::get_next_entry`.
2. `User` submits a new work order.
3. App receives a new `Entry` from `ClientProxy::get_next_entry`.
4. App notifies the machine to start counting/measuring and provides bounds from the `Entry`.
5. App polls `ClientProxy::get_scrap_quantity` to detect changes in scrap quantity.
6. `User` submits the scrap quantity.
7. App detects the change in scrap quantity and notifies the machine to stop counting/measuring.
8. App finalizes the data and submits it via `ClientProxy::finalize`.
9. Repeat from step 1.