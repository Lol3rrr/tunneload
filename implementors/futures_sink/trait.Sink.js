(function() {var implementors = {};
implementors["futures_channel"] = [{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"futures_sink/trait.Sink.html\" title=\"trait futures_sink::Sink\">Sink</a>&lt;T&gt; for <a class=\"struct\" href=\"futures_channel/mpsc/struct.Sender.html\" title=\"struct futures_channel::mpsc::Sender\">Sender</a>&lt;T&gt;","synthetic":false,"types":["futures_channel::mpsc::Sender"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"futures_sink/trait.Sink.html\" title=\"trait futures_sink::Sink\">Sink</a>&lt;T&gt; for <a class=\"struct\" href=\"futures_channel/mpsc/struct.UnboundedSender.html\" title=\"struct futures_channel::mpsc::UnboundedSender\">UnboundedSender</a>&lt;T&gt;","synthetic":false,"types":["futures_channel::mpsc::UnboundedSender"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"futures_sink/trait.Sink.html\" title=\"trait futures_sink::Sink\">Sink</a>&lt;T&gt; for &amp;<a class=\"struct\" href=\"futures_channel/mpsc/struct.UnboundedSender.html\" title=\"struct futures_channel::mpsc::UnboundedSender\">UnboundedSender</a>&lt;T&gt;","synthetic":false,"types":["futures_channel::mpsc::UnboundedSender"]}];
implementors["futures_sink"] = [];
implementors["tokio_util"] = [{"text":"impl&lt;T, I, U&gt; <a class=\"trait\" href=\"futures_sink/trait.Sink.html\" title=\"trait futures_sink::Sink\">Sink</a>&lt;I&gt; for <a class=\"struct\" href=\"tokio_util/codec/struct.Framed.html\" title=\"struct tokio_util::codec::Framed\">Framed</a>&lt;T, U&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"tokio/io/async_write/trait.AsyncWrite.html\" title=\"trait tokio::io::async_write::AsyncWrite\">AsyncWrite</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;U: <a class=\"trait\" href=\"tokio_util/codec/trait.Encoder.html\" title=\"trait tokio_util::codec::Encoder\">Encoder</a>&lt;I&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;U::<a class=\"type\" href=\"tokio_util/codec/trait.Encoder.html#associatedtype.Error\" title=\"type tokio_util::codec::Encoder::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/io/error/struct.Error.html\" title=\"struct std::io::error::Error\">Error</a>&gt;,&nbsp;</span>","synthetic":false,"types":["tokio_util::codec::framed::Framed"]},{"text":"impl&lt;T, I, D&gt; <a class=\"trait\" href=\"futures_sink/trait.Sink.html\" title=\"trait futures_sink::Sink\">Sink</a>&lt;I&gt; for <a class=\"struct\" href=\"tokio_util/codec/struct.FramedRead.html\" title=\"struct tokio_util::codec::FramedRead\">FramedRead</a>&lt;T, D&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"futures_sink/trait.Sink.html\" title=\"trait futures_sink::Sink\">Sink</a>&lt;I&gt;,&nbsp;</span>","synthetic":false,"types":["tokio_util::codec::framed_read::FramedRead"]},{"text":"impl&lt;T, I, E&gt; <a class=\"trait\" href=\"futures_sink/trait.Sink.html\" title=\"trait futures_sink::Sink\">Sink</a>&lt;I&gt; for <a class=\"struct\" href=\"tokio_util/codec/struct.FramedWrite.html\" title=\"struct tokio_util::codec::FramedWrite\">FramedWrite</a>&lt;T, E&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"tokio/io/async_write/trait.AsyncWrite.html\" title=\"trait tokio::io::async_write::AsyncWrite\">AsyncWrite</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;E: <a class=\"trait\" href=\"tokio_util/codec/trait.Encoder.html\" title=\"trait tokio_util::codec::Encoder\">Encoder</a>&lt;I&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;E::<a class=\"type\" href=\"tokio_util/codec/trait.Encoder.html#associatedtype.Error\" title=\"type tokio_util::codec::Encoder::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/io/error/struct.Error.html\" title=\"struct std::io::error::Error\">Error</a>&gt;,&nbsp;</span>","synthetic":false,"types":["tokio_util::codec::framed_write::FramedWrite"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()