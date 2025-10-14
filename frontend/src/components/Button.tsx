import type { ButtonHTMLAttributes } from "react";
import { twMerge } from "tailwind-merge";

export default function Button({
    className,
    children,
    ...rest
}: ButtonHTMLAttributes<HTMLButtonElement>) {
    let defaultClass =
        "bg-shis-900 hover:bg-shis-800 rounded-full border border-shis-600";
    if (rest.onClick) {
        defaultClass = twMerge(defaultClass, "cursor-pointer");
    }
    if (rest.disabled) {
        defaultClass = twMerge(
            defaultClass,
            "bg-shis-700 text-shis-400 hover:bg-shis-700 cursor-default",
        );
    }
    return (
        <button className={twMerge(defaultClass, className)} {...rest}>
            {children}
        </button>
    );
}
