export function Greeting({ name }: { name: string }) @{
    const greeting = `Hello ${name}`;
    <h1>{greeting}</h1>
}
