import { useState } from "react";
import LoadChart from "./LoadChart";
import Tomada, { type TomadaState } from "../dispositivos/Tomada";

type TomadaStates = {
    econ: boolean;
    state: TomadaState;
    id: string;
}[];

export default function Demo() {
    const [estados, setEstados] = useState<TomadaStates>(
        Array.from({ length: 3 }).map(() => ({
            econ: false,
            state: "on",
            id: crypto.randomUUID(),
        })),
    );
    const tomadas = estados.map((s, i) => {
        const handler = (e: boolean) => {
            setEstados(
                estados.map((s, idx) => {
                    if (idx === i) {
                        s.econ = e;
                    }
                    return s;
                }),
            );
        };
        console.log(s);
        return (
            <Tomada
                key={s.id}
                name={`Tomada ${i + 1}`}
                id={"..."}
                economy={s.econ}
                onChangeEcon={handler}
                state={s.state}
                company={"goodwe"}
                dummy
            />
        );
    });

    const econHandler = (e: boolean) => {
        setEstados(
            estados.map((s) => {
                if (s.econ) {
                    s.state = e ? "off" : "on";
                }
                return s;
            }),
        );
    };

    return (
        <>
            <div className="w-full md:h-64 mb-4 p-6 border border-shis-700 rounded-2xl bg-gradient-to-b from-shis-900 to-shis-950">
                <div className="grid not-md:grid-rows-2 md:grid-cols-3 w-full h-full">
                    <div className="flex flex-col justify-between">
                        <div>
                            <h1 className="mb-1 text-xl font-semibold">
                                Simulação de economia de energia
                            </h1>
                            <p>
                                Simula como as tomadas serão ligadas e
                                desligadas ao longo do dia, dependendo da
                                configuração global e individual de economia.
                            </p>
                            <p>
                                Clique no gráfico ao lado para simular o
                                comportamento em determinado horário:
                            </p>
                        </div>
                    </div>
                    <div className="col-span-2">
                        <LoadChart onClickEcon={econHandler} />
                    </div>
                </div>
            </div>
            <div className="grid md:grid-cols-4 grid-cols-1 gap-4">
                {tomadas}
            </div>
        </>
    );
}
