import Badge from "../components/Badge";
import GoodweW from "../assets/GoodWe_W.svg";
import TuyaT from "../assets/tuya_t.svg";

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

export default function Tomada({ id, name, state, company }: TomadaProps) {
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
        <div className="flex flex-col gap-3 p-4 border rounded-2xl from-shis-950 to-shis-900 bg-gradient-to-t border-shis-700">
            <div className="flex flex-row pl-2">
                <h2
                    className="inline w-full overflow-hidden text-ellipsis whitespace-nowrap text-xl font-semibold"
                    title={name}
                >
                    {name}
                </h2>
                <img src={tomadasImg[company]} className="inline h-8" />
            </div>
            <div className="h-36 p-2 border rounded-lg border-shis-700 bg-shis-800">
                <img src="/tomada.webp" className="h-full mx-auto" />
            </div>
            <div>
                <div className="grid grid-cols-2 gap-2">
                    <Badge
                        text={stateStr[state] ?? "caralho"}
                        dotColor={col}
                        className="w-full"
                    />
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
