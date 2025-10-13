import { Link } from "react-router";
import Button from "./components/Button";

export default function Fallback() {
    return (
        <div className="flex flex-col gap-2">
            <h1 className="text-center text-4xl font-bold">404</h1>
            <p className="text-center">Página não encontrada</p>
            <Link className="mx-auto" to="/">
                <Button className="py-2 px-6">
                    Voltar
                </Button>
            </Link>
        </div>
    );
}
