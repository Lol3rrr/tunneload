(function() {var implementors = {};
implementors["futures_channel"] = [{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"futures_channel/oneshot/struct.Cancellation.html\" title=\"struct futures_channel::oneshot::Cancellation\">Cancellation</a>&lt;'_, T&gt;","synthetic":false,"types":["futures_channel::oneshot::Cancellation"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"futures_channel/oneshot/struct.Receiver.html\" title=\"struct futures_channel::oneshot::Receiver\">Receiver</a>&lt;T&gt;","synthetic":false,"types":["futures_channel::oneshot::Receiver"]}];
implementors["futures_task"] = [{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"futures_task/struct.LocalFutureObj.html\" title=\"struct futures_task::LocalFutureObj\">LocalFutureObj</a>&lt;'_, T&gt;","synthetic":false,"types":["futures_task::future_obj::LocalFutureObj"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"futures_task/struct.FutureObj.html\" title=\"struct futures_task::FutureObj\">FutureObj</a>&lt;'_, T&gt;","synthetic":false,"types":["futures_task::future_obj::FutureObj"]}];
implementors["h2"] = [{"text":"impl&lt;B&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"h2/client/struct.ReadySendRequest.html\" title=\"struct h2::client::ReadySendRequest\">ReadySendRequest</a>&lt;B&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;B: <a class=\"trait\" href=\"bytes/buf/buf_impl/trait.Buf.html\" title=\"trait bytes::buf::buf_impl::Buf\">Buf</a> + 'static,&nbsp;</span>","synthetic":false,"types":["h2::client::ReadySendRequest"]},{"text":"impl&lt;T, B&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"h2/client/struct.Connection.html\" title=\"struct h2::client::Connection\">Connection</a>&lt;T, B&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"tokio/io/async_read/trait.AsyncRead.html\" title=\"trait tokio::io::async_read::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"tokio/io/async_write/trait.AsyncWrite.html\" title=\"trait tokio::io::async_write::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;B: <a class=\"trait\" href=\"bytes/buf/buf_impl/trait.Buf.html\" title=\"trait bytes::buf::buf_impl::Buf\">Buf</a> + 'static,&nbsp;</span>","synthetic":false,"types":["h2::client::Connection"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"h2/client/struct.ResponseFuture.html\" title=\"struct h2::client::ResponseFuture\">ResponseFuture</a>","synthetic":false,"types":["h2::client::ResponseFuture"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"h2/client/struct.PushedResponseFuture.html\" title=\"struct h2::client::PushedResponseFuture\">PushedResponseFuture</a>","synthetic":false,"types":["h2::client::PushedResponseFuture"]},{"text":"impl&lt;T, B:&nbsp;<a class=\"trait\" href=\"bytes/buf/buf_impl/trait.Buf.html\" title=\"trait bytes::buf::buf_impl::Buf\">Buf</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"h2/server/struct.Handshake.html\" title=\"struct h2::server::Handshake\">Handshake</a>&lt;T, B&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"tokio/io/async_read/trait.AsyncRead.html\" title=\"trait tokio::io::async_read::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"tokio/io/async_write/trait.AsyncWrite.html\" title=\"trait tokio::io::async_write::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;B: <a class=\"trait\" href=\"bytes/buf/buf_impl/trait.Buf.html\" title=\"trait bytes::buf::buf_impl::Buf\">Buf</a> + 'static,&nbsp;</span>","synthetic":false,"types":["h2::server::Handshake"]}];
implementors["http_body"] = [{"text":"impl&lt;'a, T:&nbsp;<a class=\"trait\" href=\"http_body/trait.Body.html\" title=\"trait http_body::Body\">Body</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"http_body/struct.Data.html\" title=\"struct http_body::Data\">Data</a>&lt;'a, T&gt;","synthetic":false,"types":["http_body::next::Data"]},{"text":"impl&lt;'a, T:&nbsp;<a class=\"trait\" href=\"http_body/trait.Body.html\" title=\"trait http_body::Body\">Body</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"http_body/struct.Trailers.html\" title=\"struct http_body::Trailers\">Trailers</a>&lt;'a, T&gt;","synthetic":false,"types":["http_body::next::Trailers"]}];
implementors["hyper"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"hyper/upgrade/struct.OnUpgrade.html\" title=\"struct hyper::upgrade::OnUpgrade\">OnUpgrade</a>","synthetic":false,"types":["hyper::upgrade::OnUpgrade"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"hyper/client/connect/dns/struct.GaiFuture.html\" title=\"struct hyper::client::connect::dns::GaiFuture\">GaiFuture</a>","synthetic":false,"types":["hyper::client::connect::dns::GaiFuture"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"hyper/client/struct.ResponseFuture.html\" title=\"struct hyper::client::ResponseFuture\">ResponseFuture</a>","synthetic":false,"types":["hyper::client::client::ResponseFuture"]},{"text":"impl&lt;T, B&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"hyper/client/conn/struct.Connection.html\" title=\"struct hyper::client::conn::Connection\">Connection</a>&lt;T, B&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"tokio/io/async_read/trait.AsyncRead.html\" title=\"trait tokio::io::async_read::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"tokio/io/async_write/trait.AsyncWrite.html\" title=\"trait tokio::io::async_write::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;B: <a class=\"trait\" href=\"hyper/body/trait.HttpBody.html\" title=\"trait hyper::body::HttpBody\">HttpBody</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;B::<a class=\"type\" href=\"hyper/body/trait.HttpBody.html#associatedtype.Data\" title=\"type hyper::body::HttpBody::Data\">Data</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;B::<a class=\"type\" href=\"hyper/body/trait.HttpBody.html#associatedtype.Error\" title=\"type hyper::body::HttpBody::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html\" title=\"struct alloc::boxed::Box\">Box</a>&lt;dyn <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/error/trait.Error.html\" title=\"trait std::error::Error\">StdError</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>&gt;&gt;,&nbsp;</span>","synthetic":false,"types":["hyper::client::conn::Connection"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"hyper/client/conn/struct.ResponseFuture.html\" title=\"struct hyper::client::conn::ResponseFuture\">ResponseFuture</a>","synthetic":false,"types":["hyper::client::conn::ResponseFuture"]}];
implementors["hyper_tls"] = [{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"tokio/io/async_read/trait.AsyncRead.html\" title=\"trait tokio::io::async_read::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"tokio/io/async_write/trait.AsyncWrite.html\" title=\"trait tokio::io::async_write::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"hyper_tls/struct.HttpsConnecting.html\" title=\"struct hyper_tls::HttpsConnecting\">HttpsConnecting</a>&lt;T&gt;","synthetic":false,"types":["hyper_tls::client::HttpsConnecting"]}];
implementors["kube_runtime"] = [{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"kube_runtime/utils/struct.CancelableJoinHandle.html\" title=\"struct kube_runtime::utils::CancelableJoinHandle\">CancelableJoinHandle</a>&lt;T&gt;","synthetic":false,"types":["kube_runtime::utils::CancelableJoinHandle"]}];
implementors["snafu"] = [{"text":"impl&lt;Fut, C, E&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"snafu/futures/try_future/struct.Context.html\" title=\"struct snafu::futures::try_future::Context\">Context</a>&lt;Fut, C, E&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Fut: <a class=\"trait\" href=\"futures_core/future/trait.TryFuture.html\" title=\"trait futures_core::future::TryFuture\">TryFuture</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;C: <a class=\"trait\" href=\"snafu/trait.IntoError.html\" title=\"trait snafu::IntoError\">IntoError</a>&lt;E, Source = Fut::<a class=\"type\" href=\"futures_core/future/trait.TryFuture.html#associatedtype.Error\" title=\"type futures_core::future::TryFuture::Error\">Error</a>&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> + <a class=\"trait\" href=\"snafu/trait.ErrorCompat.html\" title=\"trait snafu::ErrorCompat\">ErrorCompat</a>,&nbsp;</span>","synthetic":false,"types":["snafu::futures::try_future::Context"]},{"text":"impl&lt;Fut, F, C, E&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"snafu/futures/try_future/struct.WithContext.html\" title=\"struct snafu::futures::try_future::WithContext\">WithContext</a>&lt;Fut, F, E&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Fut: <a class=\"trait\" href=\"futures_core/future/trait.TryFuture.html\" title=\"trait futures_core::future::TryFuture\">TryFuture</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;F: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html\" title=\"trait core::ops::function::FnOnce\">FnOnce</a>() -&gt; C,<br>&nbsp;&nbsp;&nbsp;&nbsp;C: <a class=\"trait\" href=\"snafu/trait.IntoError.html\" title=\"trait snafu::IntoError\">IntoError</a>&lt;E, Source = Fut::<a class=\"type\" href=\"futures_core/future/trait.TryFuture.html#associatedtype.Error\" title=\"type futures_core::future::TryFuture::Error\">Error</a>&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> + <a class=\"trait\" href=\"snafu/trait.ErrorCompat.html\" title=\"trait snafu::ErrorCompat\">ErrorCompat</a>,&nbsp;</span>","synthetic":false,"types":["snafu::futures::try_future::WithContext"]}];
implementors["tokio"] = [{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio/task/struct.JoinHandle.html\" title=\"struct tokio::task::JoinHandle\">JoinHandle</a>&lt;T&gt;","synthetic":false,"types":["tokio::runtime::task::join::JoinHandle"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio/sync/futures/struct.Notified.html\" title=\"struct tokio::sync::futures::Notified\">Notified</a>&lt;'_&gt;","synthetic":false,"types":["tokio::sync::notify::Notified"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio/sync/oneshot/struct.Receiver.html\" title=\"struct tokio::sync::oneshot::Receiver\">Receiver</a>&lt;T&gt;","synthetic":false,"types":["tokio::sync::oneshot::Receiver"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio/task/struct.LocalSet.html\" title=\"struct tokio::task::LocalSet\">LocalSet</a>","synthetic":false,"types":["tokio::task::local::LocalSet"]},{"text":"impl&lt;F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio/task/struct.Unconstrained.html\" title=\"struct tokio::task::Unconstrained\">Unconstrained</a>&lt;F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>,&nbsp;</span>","synthetic":false,"types":["tokio::task::unconstrained::Unconstrained"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio/time/struct.Sleep.html\" title=\"struct tokio::time::Sleep\">Sleep</a>","synthetic":false,"types":["tokio::time::driver::sleep::Sleep"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio/time/struct.Timeout.html\" title=\"struct tokio::time::Timeout\">Timeout</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>,&nbsp;</span>","synthetic":false,"types":["tokio::time::timeout::Timeout"]}];
implementors["tokio_rustls"] = [{"text":"impl&lt;IO:&nbsp;<a class=\"trait\" href=\"tokio/io/async_read/trait.AsyncRead.html\" title=\"trait tokio::io::async_read::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"tokio/io/async_write/trait.AsyncWrite.html\" title=\"trait tokio::io::async_write::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio_rustls/struct.Connect.html\" title=\"struct tokio_rustls::Connect\">Connect</a>&lt;IO&gt;","synthetic":false,"types":["tokio_rustls::Connect"]},{"text":"impl&lt;IO:&nbsp;<a class=\"trait\" href=\"tokio/io/async_read/trait.AsyncRead.html\" title=\"trait tokio::io::async_read::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"tokio/io/async_write/trait.AsyncWrite.html\" title=\"trait tokio::io::async_write::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio_rustls/struct.Accept.html\" title=\"struct tokio_rustls::Accept\">Accept</a>&lt;IO&gt;","synthetic":false,"types":["tokio_rustls::Accept"]},{"text":"impl&lt;IO:&nbsp;<a class=\"trait\" href=\"tokio/io/async_read/trait.AsyncRead.html\" title=\"trait tokio::io::async_read::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"tokio/io/async_write/trait.AsyncWrite.html\" title=\"trait tokio::io::async_write::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio_rustls/struct.FailableConnect.html\" title=\"struct tokio_rustls::FailableConnect\">FailableConnect</a>&lt;IO&gt;","synthetic":false,"types":["tokio_rustls::FailableConnect"]},{"text":"impl&lt;IO:&nbsp;<a class=\"trait\" href=\"tokio/io/async_read/trait.AsyncRead.html\" title=\"trait tokio::io::async_read::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"tokio/io/async_write/trait.AsyncWrite.html\" title=\"trait tokio::io::async_write::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio_rustls/struct.FailableAccept.html\" title=\"struct tokio_rustls::FailableAccept\">FailableAccept</a>&lt;IO&gt;","synthetic":false,"types":["tokio_rustls::FailableAccept"]}];
implementors["tokio_util"] = [{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio_util/sync/struct.WaitForCancellationFuture.html\" title=\"struct tokio_util::sync::WaitForCancellationFuture\">WaitForCancellationFuture</a>&lt;'a&gt;","synthetic":false,"types":["tokio_util::sync::cancellation_token::WaitForCancellationFuture"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tokio_util/sync/struct.ReusableBoxFuture.html\" title=\"struct tokio_util::sync::ReusableBoxFuture\">ReusableBoxFuture</a>&lt;T&gt;","synthetic":false,"types":["tokio_util::sync::reusable_box::ReusableBoxFuture"]},{"text":"impl&lt;L, R, O&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"enum\" href=\"tokio_util/either/enum.Either.html\" title=\"enum tokio_util::either::Either\">Either</a>&lt;L, R&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;L: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&lt;Output = O&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;R: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&lt;Output = O&gt;,&nbsp;</span>","synthetic":false,"types":["tokio_util::either::Either"]}];
implementors["tower"] = [{"text":"impl&lt;F, T, E&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tower/buffer/future/struct.ResponseFuture.html\" title=\"struct tower::buffer::future::ResponseFuture\">ResponseFuture</a>&lt;F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&lt;Output = <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;T, E&gt;&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"type\" href=\"tower/type.BoxError.html\" title=\"type tower::BoxError\">BoxError</a>&gt;,&nbsp;</span>","synthetic":false,"types":["tower::buffer::future::ResponseFuture"]},{"text":"impl&lt;F1, F2:&nbsp;<a class=\"trait\" href=\"futures_core/future/trait.TryFuture.html\" title=\"trait futures_core::future::TryFuture\">TryFuture</a>, N&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tower/util/future/struct.AndThenFuture.html\" title=\"struct tower::util::future::AndThenFuture\">AndThenFuture</a>&lt;F1, F2, N&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;AndThen&lt;ErrInto&lt;F1, F2::<a class=\"type\" href=\"futures_core/future/trait.TryFuture.html#associatedtype.Error\" title=\"type futures_core::future::TryFuture::Error\">Error</a>&gt;, F2, N&gt;: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>,&nbsp;</span>","synthetic":false,"types":["tower::util::and_then::AndThenFuture"]},{"text":"impl&lt;A, B, T, AE, BE&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"enum\" href=\"tower/util/enum.Either.html\" title=\"enum tower::util::Either\">Either</a>&lt;A, B&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;A: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&lt;Output = <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;T, AE&gt;&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;AE: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"type\" href=\"tower/type.BoxError.html\" title=\"type tower::BoxError\">BoxError</a>&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;B: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&lt;Output = <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;T, BE&gt;&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;BE: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"type\" href=\"tower/type.BoxError.html\" title=\"type tower::BoxError\">BoxError</a>&gt;,&nbsp;</span>","synthetic":false,"types":["tower::util::either::Either"]},{"text":"impl&lt;F, N&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tower/util/future/struct.MapErrFuture.html\" title=\"struct tower::util::future::MapErrFuture\">MapErrFuture</a>&lt;F, N&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;MapErr&lt;F, N&gt;: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>,&nbsp;</span>","synthetic":false,"types":["tower::util::map_err::MapErrFuture"]},{"text":"impl&lt;F, N&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tower/util/future/struct.MapResponseFuture.html\" title=\"struct tower::util::future::MapResponseFuture\">MapResponseFuture</a>&lt;F, N&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;MapOk&lt;F, N&gt;: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>,&nbsp;</span>","synthetic":false,"types":["tower::util::map_response::MapResponseFuture"]},{"text":"impl&lt;F, N&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tower/util/future/struct.MapResultFuture.html\" title=\"struct tower::util::future::MapResultFuture\">MapResultFuture</a>&lt;F, N&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Map&lt;F, N&gt;: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>,&nbsp;</span>","synthetic":false,"types":["tower::util::map_result::MapResultFuture"]},{"text":"impl&lt;S, Req&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tower/util/struct.Oneshot.html\" title=\"struct tower::util::Oneshot\">Oneshot</a>&lt;S, Req&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"tower/trait.Service.html\" title=\"trait tower::Service\">Service</a>&lt;Req&gt;,&nbsp;</span>","synthetic":false,"types":["tower::util::oneshot::Oneshot"]},{"text":"impl&lt;F, T, E&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tower/util/future/optional/struct.ResponseFuture.html\" title=\"struct tower::util::future::optional::ResponseFuture\">ResponseFuture</a>&lt;F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&lt;Output = <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;T, E&gt;&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"type\" href=\"tower/type.BoxError.html\" title=\"type tower::BoxError\">BoxError</a>&gt;,&nbsp;</span>","synthetic":false,"types":["tower::util::optional::future::ResponseFuture"]},{"text":"impl&lt;T, Request&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tower/util/struct.ReadyOneshot.html\" title=\"struct tower::util::ReadyOneshot\">ReadyOneshot</a>&lt;T, Request&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"tower/trait.Service.html\" title=\"trait tower::Service\">Service</a>&lt;Request&gt;,&nbsp;</span>","synthetic":false,"types":["tower::util::ready::ReadyOneshot"]},{"text":"impl&lt;'a, T, Request&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tower/util/struct.ReadyAnd.html\" title=\"struct tower::util::ReadyAnd\">ReadyAnd</a>&lt;'a, T, Request&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"tower/trait.Service.html\" title=\"trait tower::Service\">Service</a>&lt;Request&gt;,&nbsp;</span>","synthetic":false,"types":["tower::util::ready::ReadyAnd"]},{"text":"impl&lt;F1, F2, N&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tower/util/future/struct.ThenFuture.html\" title=\"struct tower::util::future::ThenFuture\">ThenFuture</a>&lt;F1, F2, N&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Then&lt;F1, F2, N&gt;: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>,&nbsp;</span>","synthetic":false,"types":["tower::util::then::ThenFuture"]}];
implementors["tracing"] = [{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tracing/instrument/struct.Instrumented.html\" title=\"struct tracing::instrument::Instrumented\">Instrumented</a>&lt;T&gt;","synthetic":false,"types":["tracing::instrument::Instrumented"]}];
implementors["tracing_futures"] = [{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tracing_futures/struct.Instrumented.html\" title=\"struct tracing_futures::Instrumented\">Instrumented</a>&lt;T&gt;","synthetic":false,"types":["tracing_futures::Instrumented"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"tracing_futures/struct.WithDispatch.html\" title=\"struct tracing_futures::WithDispatch\">WithDispatch</a>&lt;T&gt;","synthetic":false,"types":["tracing_futures::WithDispatch"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()