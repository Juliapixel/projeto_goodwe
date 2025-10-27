import {
    Bar,
    BarChart,
    CartesianGrid,
    Label,
    Legend,
    Line,
    LineChart,
    ResponsiveContainer,
    Tooltip,
    XAxis,
    YAxis,
    type TooltipContentProps,
} from "recharts";
import Header from "./components/Header";
import Footer from "./components/Footer";
import Membros from "./components/Membros";
import type React from "react";
import {
    useEffect,
    useRef,
    useState,
    type HTMLAttributes,
    type Ref,
} from "react";
import { twMerge } from "tailwind-merge";
import { getDados, getEconomiaChart } from "./lib";
import Button from "./components/Button";
import { Link } from "react-router";
import TuyaT from "./assets/tuya_t.svg";
import Tomada from "./dispositivos/Tomada";

function tooltipRender(props: TooltipContentProps<number, string>) {
    const values = props.payload.map((p) => {
        return (
            <div>
                <span style={{ color: p.color }}>{p.name}</span>{" "}
                <span>{Math.round(p.value * 100) / 100}</span>
            </div>
        );
    });
    return (
        <div className="bg-zinc-800/40 border border-zinc-300/10 rounded-lg p-2 backdrop-blur-sm">
            <p>{props.label}</p>
            {values}
        </div>
    );
}

function MultiChart() {
    const [data, setData] =
        useState<Awaited<ReturnType<typeof getDados>>>(undefined);
    useEffect(() => {
        getDados().then((i) => setData(i));
    }, []);
    return (
        <ResponsiveContainer>
            <LineChart data={data}>
                <CartesianGrid stroke="#ffffff30" strokeDasharray={"3 3"} />
                <Line
                    dataKey={"load"}
                    name="Consumo (W)"
                    stroke="lime"
                    dot={false}
                    strokeWidth={2}
                />
                <Line
                    dataKey={"pv"}
                    name="Geração (W)"
                    stroke="orange"
                    dot={false}
                    strokeWidth={2}
                />
                <XAxis dataKey={"date"} stroke="#ffffff90" />
                <YAxis stroke="#ffffff90" />
                <Legend align="right" />
                <Tooltip animationDuration={100} content={tooltipRender} />
                {data ? (
                    <></>
                ) : (
                    <Label
                        className="select-none"
                        textAnchor="middle"
                        position={"center"}
                        fill="white"
                        fontSize="200%"
                        fontWeight={600}
                    >
                        Carregando...
                    </Label>
                )}
            </LineChart>
        </ResponsiveContainer>
    );
}

function Economia() {
    const [data, setData] =
        useState<Awaited<ReturnType<typeof getEconomiaChart>>>(undefined);
    useEffect(() => {
        getEconomiaChart().then((i) => setData(i));
    }, []);
    return (
        <ResponsiveContainer>
            <BarChart data={data}>
                <CartesianGrid stroke="#ffffff30" strokeDasharray={"3 3"} />
                <XAxis dataKey={0} stroke="#ffffff90" />
                <YAxis dataKey={1} stroke="#ffffff90" />
                <Legend align="right" />
                <Tooltip animationDuration={100} content={tooltipRender} />
                <Bar dataKey={1} name={"Economia (kWh)"} fill="red" />
                {data ? (
                    <></>
                ) : (
                    <Label
                        className="select-none"
                        textAnchor="middle"
                        position={"center"}
                        fill="white"
                        fontSize="200%"
                        fontWeight={600}
                    >
                        Carregando...
                    </Label>
                )}
            </BarChart>
        </ResponsiveContainer>
    );
}

function Slide({
    children,
    className,
    ref,
    ...rest
}: {
    children: React.ReactNode;
    ref?: Ref<HTMLDivElement>;
} & HTMLAttributes<HTMLDivElement>) {
    return (
        <div
            ref={ref}
            className={twMerge(
                "flex h-screen w-screen pt-48 pb-32 px-20 border-y border-shis-800",
                className,
            )}
            {...rest}
        >
            {children}
        </div>
    );
}

export default function Home() {
    // efeito de slide muito epico
    const slide1 = useRef<HTMLDivElement>(null);
    const slide2 = useRef<HTMLDivElement>(null);
    const slide3 = useRef<HTMLDivElement>(null);
    const slide4 = useRef<HTMLDivElement>(null);
    const slides = [slide1, slide2, slide3, slide4];
    const slidePos = useRef(0);
    useEffect(() => {
        const scrollHandler = (e) => {
            if (e.key == "PageDown") {
                e.preventDefault();
                slidePos.current += 1;
                slidePos.current = slidePos.current % slides.length;
                slides[slidePos.current].current?.scrollIntoView({
                    behavior: "smooth",
                });
            } else if (e.key == "PageUp") {
                e.preventDefault();
                slidePos.current -= 1;
                slidePos.current =
                    (slidePos.current + slides.length) % slides.length;
                slides[slidePos.current].current?.scrollIntoView({
                    behavior: "smooth",
                });
            }
        };
        window.addEventListener("keydown", scrollHandler);
        return () => window.removeEventListener("keydown", scrollHandler);
    });

    return (
        <main>
            <Header />
            <Slide ref={slide1} className="bg-gradient-to-b to-shis-800">
                <div className="m-auto">
                    <h1 className="mb-6 text-center text-6xl font-bold">
                        Solar House Integrated Systems
                    </h1>
                    <p className="mb-24 text-center text-2xl">
                        1CCPF - Equipe 3
                    </p>
                    <Membros className="font-medium text-xl" />
                </div>
            </Slide>
            <Slide ref={slide2} className="bg-zinc-100 text-black">
                <div className="grid grid-cols-2 gap-12">
                    <div className="my-auto">
                        <h1 className="text-5xl font-bold mb-4">
                            A sua solução de economia energética residencial
                        </h1>
                        <p className="text-3xl">
                            Terrível contra o standby. Contra o standby.
                        </p>
                    </div>
                    <div className="my-auto text-2xl font-medium">
                        <ul className="list-disc mb-12">
                            <li>
                                Se integra com o seu sistema de inversores e
                                baterias GoodWe&reg;
                            </li>
                            <li>
                                Tomadas inteligentes automaticamente desligam os
                                seus dispositivos em standby quando não estiver
                                usando
                            </li>
                            <li>
                                Economiza seu sistema de Backup da GoodWe&reg;
                                quando a concessionária estiver fora do ar
                            </li>
                        </ul>
                        <div className="w-96">
                            <p className="mb-4">Integração com Alexa&trade;</p>
                            <img
                                className="ml-2"
                                src="/Amazon_Alexa_logo.svg"
                            />
                        </div>
                    </div>
                </div>
            </Slide>
            <Slide ref={slide3} className="grid grid-cols-2 gap-12">
                <div className="my-auto">
                    <h1 className="mb-6 text-5xl font-bold">
                        Sistema de Aprendizado de Máquina
                    </h1>
                    <p className="mb-4 text-2xl">
                        Se adapta à sua rotina diária e controla tudo para você.
                    </p>
                    <Link to="/demo">
                        <Button className="px-8 py-4 underline text-2xl font-medium cursor-pointer">
                            Simulação
                        </Button>
                    </Link>
                </div>
                <div className="p-12 bg-shis-900 rounded-2xl border border-shis-700">
                    <h2 className="text-xl font-medium">Dados reais:</h2>
                    <div className="h-full">
                        <div className="h-1/2">
                            <Economia />
                        </div>
                        <div className="h-1/2">
                            <MultiChart />
                        </div>
                    </div>
                </div>
            </Slide>
            <Slide
                ref={slide4}
                className="grid grid-cols-2 gap-12 bg-gradient-to-b to-[#e60012] from-60%"
            >
                <div className="my-auto">
                    <h1 className="mb-12 text-5xl font-bold">
                        Tomadas Inteligentes Integradas
                    </h1>
                    <div className="mb-8 flex gap-8 text-2xl">
                        <img
                            src="/rust-lang-logo-black.svg"
                            className="inline h-24 select-none"
                        />
                        <div className="my-auto">
                            <p>Tomada customizada própria:</p>
                            <ul className="list-disc list-inside">
                                <li>Feita em Rust</li>
                                <li>Controle remoto por Alexa&trade;</li>
                            </ul>
                        </div>
                    </div>
                    <div className="flex gap-8 text-2xl">
                        <img
                            src={TuyaT}
                            className="inline h-24 rounded-xl select-none"
                        />
                        <div className="my-auto">
                            <p>Tomadas Tuya&trade;:</p>
                            <ul className="list-disc list-inside">
                                <li>Leitura remota de consumo</li>
                                <li>Controle remoto pelo app Tuya&trade;</li>
                            </ul>
                        </div>
                    </div>
                </div>
                <div className="w-96 m-auto">
                    <Tomada
                        className="mb-12 shadow-2xl shadow-red-900"
                        id="..."
                        company="goodwe"
                        name="Tomada customizada"
                        economy={false}
                        state="off"
                    />
                    <div className="bg-zinc-200 p-8 border rounded-xl shadow-2xl shadow-blue-400">
                        <img
                            className="select-none"
                            src="/Amazon_Alexa_logo.svg"
                        />
                    </div>
                </div>
            </Slide>
            <Footer />
        </main>
    );
}
