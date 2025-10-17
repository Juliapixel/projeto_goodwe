import Tomada, { type TomadaCompany, type TomadaState } from "./Tomada";

const tomadas = Array.from({ length: 10 }, () => {
    return {
        id: crypto.randomUUID(),
        state: (["on", "off", "unknown"] as TomadaState[])[
            Math.round(Math.random() * 2)
        ],
        company: (["tuya", "goodwe"] as TomadaCompany[])[
            Math.round(Math.random())
        ],
    };
});

// UUID da tomada de testes
tomadas[0] = {
    id: "338c1c8a-c3a2-4715-be92-8911248bbb8c",
    company: "goodwe",
    state: "off",
};

export default function Dispositivos() {
    const t = tomadas.map((d, i) => (
        <Tomada
            state={d.state}
            id={d.id}
            key={d.id}
            name={`Tomada ${i + 1}`}
            company={d.company}
        />
    ));
    return (
        <>
            <div className="grid md:grid-cols-4 grid-cols-1 gap-4">{t}</div>
        </>
    );
}
