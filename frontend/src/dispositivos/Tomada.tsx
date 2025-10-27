import Badge from "../components/Badge";
import GoodweW from "../assets/GoodWe_W.svg";
import TuyaT from "../assets/tuya_t.svg";
import Button from "../components/Button";
import {
    useEffect,
    useState,
    type HTMLAttributes,
    type MouseEventHandler,
} from "react";
import { setTomada } from "../lib";
import { twMerge } from "tailwind-merge";

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

export interface TomadaProps extends HTMLAttributes<HTMLDivElement> {
    id: string;
    name: string;
    state: TomadaState;
    company: TomadaCompany;
    economy: boolean;
    load?: number;
    dummy?: boolean;
    onTogglePower?(isOn: boolean): void;
    onChangeEcon?(isOn: boolean): void;
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
    return (
        <Badge
            text={stateStr[state] ?? "caralho"}
            dotColor={col}
            className="w-full"
        />
    );
}

export default function Tomada({
    id,
    name,
    state,
    economy,
    company,
    load,
    dummy,
    className,
    onChangeEcon,
    onTogglePower,
    ...restProps
}: TomadaProps) {
    const [localState, setState] = useState(state);
    const [isOn, setIsOn] = useState(
        state == "on" ? true : state == "off" ? false : undefined,
    );
    // incrivel
    useEffect(() => {
        setState(state);
        setIsOn(state == "on" ? true : state == "off" ? false : undefined);
    }, [state]);
    const toggle: MouseEventHandler<HTMLButtonElement> = async (e) => {
        if (dummy) {
            setIsOn(!isOn);
            setState(isOn ? "off" : "on");
            onTogglePower?.(!isOn);
            return;
        }
        const target = e.currentTarget;
        target.disabled = true;
        try {
            const success = await setTomada(!isOn);
            if (success) {
                setIsOn(!isOn);
                setState(isOn ? "off" : "on");
                onTogglePower?.(!isOn);
            } else {
                setIsOn(undefined);
                setState("unknown");
            }
        } catch (error) {
            setIsOn(undefined);
            setState("unknown");
            console.error(error);
        } finally {
            target.disabled = false;
        }
    };
    return (
        <div
            className={twMerge(
                "flex flex-col gap-3 p-4 border rounded-2xl from-shis-950 to-shis-900 bg-gradient-to-t border-shis-700",
                className,
            )}
            {...restProps}
        >
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
                <p className="px-3 py-2 text-lg">
                    Consumo: {load?.toString() ?? "0.0"}W
                </p>
                <div className="grid grid-cols-2 gap-2">
                    <StatusBadge state={localState} />
                    <Button onClick={toggle} disabled={isOn === undefined}>
                        {isOn === undefined
                            ? "Offline"
                            : isOn
                              ? "Desligar"
                              : "Ligar"}
                    </Button>
                    <span className="my-auto text-center">
                        Economia de Energia
                    </span>
                    <select
                        defaultValue={economy.toString()}
                        className="h-10 text-center rounded-full border border-shis-600 bg-shis-900"
                        onChange={(e) =>
                            onChangeEcon?.(e.currentTarget.value === "true")
                        }
                    >
                        <option value={"true"}>Ligada</option>
                        <option value={"false"}>Desligada</option>
                    </select>
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
