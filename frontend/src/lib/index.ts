export const API_BASE = import.meta.env.DEV
    ? "https://goodwe.juliapixel.com"
    : "";

export async function setTomada(on: boolean) {
    const resp = await fetch(
        `${API_BASE}/api/tomada/set?state=${on ? "on" : "off"}`,
        { method: "POST" },
    );
    const data = await resp.json();
    return data.present === true;
}

export async function getTomada() {
    const resp = await fetch(`${API_BASE}/api/tomada/get`);
    const data = await resp.json();
    return data.state != null;
}

export async function setEcon(on: boolean) {
    const resp = await fetch(
        `${API_BASE}/api/tomada/set_economia?state=${on ? "on" : "off"}`,
        { method: "POST" },
    );
    return resp.ok;
}

export async function getEcon() {
    const resp = await fetch(`${API_BASE}/api/tomada/get_economia`);
    const data = await resp.json();
    return data.state == "on";
}
