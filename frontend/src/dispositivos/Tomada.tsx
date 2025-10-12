import Badge from "../components/Badge";
import GoodweW from "../assets/GoodWe_W.svg";
import TuyaT from "../assets/tuya_t.svg";
import Button from "../components/Button";
import { useState } from "react";

export type TomadaState = "on" | "off" | "unknown";
export type TomadaCompany = "goodwe" | "tuya";

const tomadasImg: Record<TomadaCompany, string> = {
    goodwe: GoodweW,
    tuya: TuyaT,
};

const stateStr: Record<TomadaState, string> = {
    on: "Ligada",
    off: "Desligada",
    unknown: "Offline",
};

export interface TomadaProps {
    id: string;
    name?: string;
    state: TomadaState;
    company: TomadaCompany;
}

function StatusBadge({ state }: { state: TomadaState }) {
    let col: string;
    switch (state) {
        case "on":
            col = "lime";
            break;
        case "off":
            col = "red";
            break;
        case "unknown":
            col = "yellow";
            break;
        default:
            col = "magenta";
            break;
    }
    return <Badge text={stateStr[state] ?? "caralho"} dotColor={col} className="w-full" />
}

export default function Tomada({ id, name, state, company }: TomadaProps) {
    const [isOn, setIsOn] = useState(state == "on" ? true : state == "off" ? false : undefined)
    const [localState, setState] = useState(state)
    const toggle = () => {
        setIsOn(!isOn);
        setState(isOn ? "off" : "on")
    }
    return (
        <div className="flex flex-col gap-3 p-4 border rounded-2xl from-shis-950 to-shis-900 bg-gradient-to-t border-shis-700">
            <div className="flex flex-row pl-2">
                <h2
                    className="inline w-full overflow-hidden text-ellipsis whitespace-nowrap text-xl font-semibold"
                    title={name}
                >
                    {name}
                </h2>
                <img
                    src={tomadasImg[company]}
                    className="inline h-8 pointer-events-none select-none"
                />
            </div>
            <div className="h-36 p-2 border rounded-lg border-shis-700 bg-shis-800">
                <img
                    src="/tomada.webp"
                    alt="Tomada inteligente"
                    className="h-full mx-auto pointer-events-none select-none"
                />
            </div>
            <div>
                <div className="grid grid-cols-2 gap-2">
                    <StatusBadge state={localState} />
                    <Button onClick={toggle} disabled={isOn === undefined}>
                        {isOn === undefined ? "Offline" : isOn ? "Desligar" : "Ligar"}
                    </Button>
                </div>
                <p
                    className="w-full pt-2 text-xs text-shis-300/50 overflow-hidden text-ellipsis whitespace-nowrap"
                    title={id}
                >
                    ID: {id}
                </p>
            </div>
        </div>
    );
}
