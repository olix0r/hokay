# hokay

A bare-bones HTTP/1 server that always returns an empty response.

This program aims to take minimal dependencies and is expected to be updated
infrequently.

## Usage

By default, `hokay` responds with `204 No Content`:

```text
:; hokay
```

```text
:; curl -v localhost:8080
*   Trying ::1:8080...
* connect to ::1 port 8080 failed: Connection refused
*   Trying 127.0.0.1:8080...
* Connected to localhost (127.0.0.1) port 8080 (#0)
> GET / HTTP/1.1
> Host: localhost:8080
> User-Agent: curl/7.74.0
> Accept: */*
>
* Mark bundle as not supporting multiuse
< HTTP/1.1 204 No Content
< server: hokay
< date: Fri, 22 Apr 2022 15:18:10 GMT
<
* Connection #0 to host localhost left intact
```

Optionally, pass `--status` to override the response's status code:

```text
hokay --status 418
```

```text
:; curl -v localhost:8080
*   Trying 127.0.0.1:8080...
* Connected to localhost (127.0.0.1) port 8080 (#0)
> GET / HTTP/1.1
> Host: localhost:8080
> User-Agent: curl/7.87.0
> Accept: */*
>
* Mark bundle as not supporting multiuse
< HTTP/1.1 418 I'm a teapot
< server: hokay/0.2.2
< content-length: 0
< date: Mon, 03 Apr 2023 19:55:49 GMT
<
* Connection #0 to host localhost left intact
```

## License

This program is distributed under the [MIT license](./LICENSE).
