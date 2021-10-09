# Valve Router

### Basic ReqRes Flow

```sequence

Client -> Valve Router: HTTP Request

Valve Router -> Valve Router: Parse request

Valve Router -> Valve Router: Lookup Attached Functions Worker

Valve Router -> Valve Router: Use Least busy worker

Valve Router -> Worker: ZMQ Request (use Function Port)

Worker -> Valve Router: ZMQ Result (use Function Port)

Valve Router -> Client: HTTP Response

```

### Worker Connecting Valve Router

```sequence

Worker -> Valve Router: 1. Connect Controller Socket

Worker -> Valve Router: 2. Send Function Config

Valve Router -> Valve Router: 3. Bind Worker's Function

Worker -> Valve Router: 4. Connect Function Port

```

1. Connect ZeroMQ server on Valve Router. (PORT: 4560)
2. Send Function Config.

```js
// sample config
{
    functionName: char[255], // Must be unique for a functionDefinition.
    timeout: int32,          // HardLimit in ms, Router will consider function failed, and send kill signal to worker
    maxConcurrency: int32    // Maxmimum Concurrent function execution the worker can handle.
}
```

3. Bind Worker's Function
> 3.1 Assing route to Function Config
> 3.2 Add Function functionWorkerMap
> 3.3 Create Function Port
> 3.4 Send updated config to function

4. Worker Connection to Function Port MQ