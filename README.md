# Tiny URL shortener service

Start everything:
```bash
docker-compose up
```

Get a short link:
```bash
curl http://localhost:8080/ --data "https://google.com"
```

Test the link:
```bash
curl http://localhost:8080/<link> -v
```
