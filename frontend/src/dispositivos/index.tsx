import { useEffect, useState } from "react";
import Tomada, { type TomadaCompany, type TomadaState } from "./Tomada";
import { getConsumo, getTomada } from "../lib";

const tomadas = Array.from({ length: 9 }, () => {
    return {
        id: crypto.randomUUID(),
        state: "unknown" as TomadaState,
        company: (["tuya", "goodwe"] as TomadaCompany[])[
            Math.round(Math.random())
        ],
        economy: false,
    };
});

export default function Dispositivos() {
    const [state, setState] = useState<TomadaState>("unknown");
    const [load, setLoad] = useState<number | undefined>();
    useEffect(() => {
        getConsumo().then((c) => setLoad(c));
        getTomada().then((c) => {
            if (c === null) setState("unknown");
            setState(c ? "on" : "off");
        });
    }, []);
    const t = tomadas.map((d, i) => (
        <Tomada
            state={d.state}
            economy={d.economy}
            id={d.id}
            key={d.id}
            name={`Tomada Futura ${i + 1}`}
            company={d.company}
        />
    ));
    return (
        <>
            <div className="grid md:grid-cols-4 grid-cols-1 gap-4">
                <Tomada
                    state={state}
                    load={load}
                    company="goodwe"
                    economy={false}
                    id="338c1c8a-c3a2-4715-be92-8911248bbb8c"
                    name="Tomada Protótipo"
                />
                {t}
            </div>
        </>
    );
}
