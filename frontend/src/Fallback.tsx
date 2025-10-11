import { Link } from "react-router";

export default function Fallback() {
    return (
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 flex flex-col gap-2">
            <h1 className="text-center text-4xl font-bold">404</h1>
            <p>Página não encontrada</p>
            <Link
                to="/"
                className="block w-full py-2 text-center border border-zinc-300/30 bg-zinc-800 rounded-full"
            >
                Voltar
            </Link>
        </div>
    );
}
