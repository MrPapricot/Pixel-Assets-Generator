import ipaddress
from fastapi import Request, HTTPException
from starlette.middleware.base import BaseHTTPMiddleware
from fastapi.responses import JSONResponse
from app.config import settings


class IPWhitelistMiddleware(BaseHTTPMiddleware):
    """Middleware для фильтрации IP адресов"""

    def __init__(self, app):
        super().__init__(app)
        self.allowed_networks = []
        for ip in settings.ALLOWED_IPS:
            if '/' in ip:
                self.allowed_networks.append(ipaddress.ip_network(ip, strict=False))
            else:
                self.allowed_networks.append(ipaddress.ip_network(f"{ip}/32"))

    async def dispatch(self, request: Request, call_next):
        # Пропускаем healthcheck и документацию
        if request.url.path in ["/health", "/docs", "/openapi.json", "/redoc"]:
            return await call_next(request)

        client_ip = request.client.host
        forwarded = request.headers.get("X-Forwarded-For")
        if forwarded:
            client_ip = forwarded.split(",")[0].strip()

        try:
            ip_obj = ipaddress.ip_address(client_ip)
            if not any(ip_obj in network for network in self.allowed_networks):
                return JSONResponse(
                    status_code=403,
                    content={"detail": f"Access denied for IP: {client_ip}"}
                )
        except ValueError:
            return JSONResponse(
                status_code=400,
                content={"detail": "Invalid IP address"}
            )

        return await call_next(request)


# Dependency для защиты конкретных эндпоинтов
def check_ip_allowed(request: Request):
    """Dependency для проверки IP"""
    client_ip = request.client.host
    forwarded = request.headers.get("X-Forwarded-For")
    if forwarded:
        client_ip = forwarded.split(",")[0].strip()

    allowed_networks = []
    for ip in settings.ALLOWED_IPS:
        if '/' in ip:
            allowed_networks.append(ipaddress.ip_network(ip, strict=False))
        else:
            allowed_networks.append(ipaddress.ip_network(f"{ip}/32"))

    try:
        ip_obj = ipaddress.ip_address(client_ip)
        if not any(ip_obj in network for network in allowed_networks):
            raise HTTPException(status_code=403, detail=f"Access denied for IP: {client_ip}")
    except ValueError:
        raise HTTPException(status_code=400, detail="Invalid IP address")

    return True