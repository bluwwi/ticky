import Keyboard from "@/components/Keyboard";
import ParrellLineBg from "@/components/Parrell-line-bg";

export default function Home() {
  return (
      <section className="flex min-h-screen items-center justify-center px-4 py-16">
        <main className="w-full max-w-5xl">
          <ParrellLineBg color="#111" bg="#fff" stroke={3}>

          <Keyboard />
        </ParrellLineBg>

        </main>
      </section>
  );
}
