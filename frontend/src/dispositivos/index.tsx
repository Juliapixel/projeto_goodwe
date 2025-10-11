import Tomada, { type TomadaCompany, type TomadaState } from "./Tomada";

export default function Dispositivos() {
    const ids = ["338c1c8a-c3a2-4715-be92-8911248bbb8c"];
    for (let i = 1; i < 10; i++) {
        ids[i] = crypto.randomUUID();
    }
    const tomadas = ids.map((d, i) => {
        return (
            <Tomada
                state={
                    (["on", "off"] as TomadaState[])[Math.round(Math.random())]
                }
                id={d}
                key={d}
                name={`Tomada ${i + 1}`}
                company={
                    (["tuya", "goodwe"] as TomadaCompany[])[
                        Math.round(Math.random())
                    ]
                }
            />
        );
    });
    return (
        <>
            <div className="grid md:grid-cols-4 grid-cols-1 gap-4">
                {tomadas}
            </div>
        </>
    );
}
