import Battery from "../components/Battery";

export default function Dados() {
    return (
        <>
            {/* <h1 className="text-6xl text-center">🎲🎲</h1> */}
            <div className="flex flex-row justify-between gap-4 w-fit mx-auto p-4 rounded-2xl bg-shis-900">
                <div>
                    <h1 className="mb-2 text-center font-bold text-2xl">Bateria</h1>
                    <div className="w-36 h-36 rounded-full border border-shis-700 bg-shis-800 ">
                        <Battery charge={96} />
                    </div>
                </div>
                <div>
                    <h1 className="mb-2 text-center font-bold text-2xl">Bateria</h1>
                    <div className="w-36 h-36 rounded-full border border-shis-700 bg-shis-800 ">
                        <Battery charge={40} />
                    </div>
                </div>
                <div>
                    <h1 className="mb-2 text-center font-bold text-2xl">Bateria</h1>
                    <div className="w-36 h-36 rounded-full border border-shis-700 bg-shis-800 ">
                        <Battery charge={10} />
                    </div>
                </div>
            </div>
        </>
    )
}
