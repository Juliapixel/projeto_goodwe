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
    const data: { state: "on" | "off" | null } = await resp.json();
    if (data.state === null) return null;
    return data.state === "on";
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

type Dado = [string, number];

interface DadosResp {
    bat: Dado[];
    charge: Dado[];
    load: Dado[];
    pv: Dado[];
    meter: Dado[];
}

export async function getDados() {
    const resp = await fetch(`${API_BASE}/api/dados`);
    if (!resp.ok) {
        return;
    }
    const data: DadosResp = await resp.json();
    return Array.from({ length: data.bat.length }).map((_, i) => {
        const date = new Date(Date.parse(data.bat[i][0]));
        return {
            date: date.toLocaleTimeString(),
            bat: data.bat[i][1],
            charge: data.charge[i][1],
            load: data.load[i][1],
            pv: data.pv[i][1],
            meter: data.meter[i][1],
        };
    });
}

export async function getEconomiaChart() {
    const resp = await fetch(`${API_BASE}/api/graficos/econ_semana`);
    if (!resp.ok) {
        return;
    }
    const data: [string, number][] = await resp.json();
    return data;
}

export async function getConsumo() {
    const resp = await fetch(`${API_BASE}/api/tuya`);
    if (!resp.ok) {
        return;
    }
    const data: number = (await resp.json()).power;
    return data;
}
