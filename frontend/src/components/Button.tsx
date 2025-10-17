import type { ButtonHTMLAttributes } from "react";
import { twMerge } from "tailwind-merge";

export default function Button({
    className,
    children,
    ...rest
}: ButtonHTMLAttributes<HTMLButtonElement>) {
    let defaultClass =
        "bg-shis-900 disabled:bg-shis-700 not-disabled:hover:bg-shis-800 rounded-full disabled:text-shis-400 border border-shis-600 disabled:cursor-default";
    if (rest.onClick) {
        defaultClass = twMerge(defaultClass, "cursor-pointer");
    }
    return (
        <button className={twMerge(defaultClass, className)} {...rest}>
            {children}
        </button>
    );
}
