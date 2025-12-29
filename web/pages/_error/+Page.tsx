import { usePageContext } from "vike-react/usePageContext";

export default function Page() {
  const { is404 } = usePageContext();
  return (
    <div className="flex flex-col items-center justify-center min-h-[50vh] text-center px-4">
      <h1 className="text-4xl font-bold mb-4">{is404 ? "Page Not Found" : "Internal Error"}</h1>
      <p className="text-gray-400">{is404 ? "This page could not be found." : "Something went wrong."}</p>
    </div>
  );
}
