initSidebarItems({"enum":[["Either","Combine two different service types into a single type."]],"fn":[["future_service","Returns a new [`FutureService`] for the given future."],["service_fn","Returns a new [`ServiceFn`] with the given closure."]],"mod":[["error","Error types"],["future","Future types"]],"struct":[["AndThen","Service returned by the `and_then` combinator."],["AndThenLayer","A `Layer` that produces a [`AndThen`] service."],["BoxService","A boxed `Service + Send` trait object."],["CallAll","This is a `Stream` of responses resulting from calling the wrapped [`Service`] for each request received on the wrapped `Stream`."],["CallAllUnordered","A stream of responses received from the inner service in received order."],["FutureService","A type that implements [`Service`] for a [`Future`] that produces a [`Service`]."],["MapErr","Service returned by the `map_err` combinator."],["MapErrLayer","A `Layer` that produces [`MapErr`] services."],["MapRequest","Service returned by the `MapRequest` combinator."],["MapRequestLayer","A `Layer` that produces [`MapRequest`] services."],["MapResponse","Service returned by the `map_response` combinator."],["MapResponseLayer","A `Layer` that produces a [`MapResponse`] service."],["MapResult","Service returned by the `map_result` combinator."],["MapResultLayer","A `Layer` that produces a [`MapResult`] service."],["Oneshot","A [`Future`] consuming a [`Service`] and request, waiting until the [`Service`] is ready, and then calling [`Service::call`] with the request, and waiting for that [`Future`]."],["Optional","Optionally forwards requests to an inner service."],["ReadyAnd","A future that yields a mutable reference to the service when it is ready to accept a request."],["ReadyOneshot","A [`Future`] that yields the service when it is ready to accept a request."],["ServiceFn","A [`Service`] implemented by a closure."],["Then","[`Service`] returned by the `then` combinator."],["ThenLayer","A `Layer` that produces a [`Then`] service."],["UnsyncBoxService","A boxed [`Service`] trait object."]],"trait":[["ServiceExt","An extension trait for `Service`s that provides a variety of convenient adapters"]]});