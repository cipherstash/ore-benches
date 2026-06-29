export const metadata = {
  title: "KMS comparison harness",
  description: "ZeroKMS vs AWS KMS load-test harness",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
