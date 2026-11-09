$csharpSource = @"
using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.IO;

public class GlastorIconGenerator
{
    public static void GeneratePng(int size, string outPath)
    {
        using (Bitmap bmp = new Bitmap(size, size))
        using (Graphics g = Graphics.FromImage(bmp))
        {
            g.SmoothingMode = SmoothingMode.AntiAlias;
            g.InterpolationMode = InterpolationMode.HighQualityBicubic;
            g.PixelOffsetMode = PixelOffsetMode.HighQuality;

            // Background
            using (SolidBrush bg = new SolidBrush(Color.FromArgb(255, 9, 9, 11)))
            {
                g.FillRectangle(bg, 0, 0, size, size);
            }

            // Squircle card
            float radius = Math.Max(2f, size * 0.22f);
            using (GraphicsPath path = new GraphicsPath())
            {
                path.AddArc(0, 0, radius * 2, radius * 2, 180, 90);
                path.AddArc(size - radius * 2 - 1, 0, radius * 2, radius * 2, 270, 90);
                path.AddArc(size - radius * 2 - 1, size - radius * 2 - 1, radius * 2, radius * 2, 0, 90);
                path.AddArc(0, size - radius * 2 - 1, radius * 2, radius * 2, 90, 90);
                path.CloseFigure();

                using (SolidBrush card = new SolidBrush(Color.FromArgb(255, 24, 24, 27)))
                {
                    g.FillPath(card, path);
                }
                using (Pen border = new Pen(Color.FromArgb(255, 39, 39, 42), Math.Max(1f, size / 32f)))
                {
                    g.DrawPath(border, path);
                }
            }

            // Layers coordinates
            float scale = size / 32f;
            float off = size * 0.125f;

            // Top polygon
            PointF[] topPoly = new PointF[] {
                new PointF((12f * scale) + off, (2f * scale) + off),
                new PointF((22f * scale) + off, (7f * scale) + off),
                new PointF((12f * scale) + off, (12f * scale) + off),
                new PointF((2f * scale) + off, (7f * scale) + off)
            };
            using (SolidBrush white = new SolidBrush(Color.White))
            {
                g.FillPolygon(white, topPoly);
            }
            using (Pen dark = new Pen(Color.FromArgb(255, 9, 9, 11), Math.Max(0.5f, scale * 0.7f)))
            {
                g.DrawPolygon(dark, topPoly);
            }

            // Middle line
            PointF[] midPts = new PointF[] {
                new PointF((2f * scale) + off, (12f * scale) + off),
                new PointF((12f * scale) + off, (17f * scale) + off),
                new PointF((22f * scale) + off, (12f * scale) + off)
            };
            using (Pen midPen = new Pen(Color.FromArgb(255, 250, 250, 250), Math.Max(1.2f, scale * 1.8f)))
            {
                midPen.StartCap = LineCap.Round;
                midPen.EndCap = LineCap.Round;
                g.DrawLines(midPen, midPts);
            }

            // Bottom line
            PointF[] botPts = new PointF[] {
                new PointF((2f * scale) + off, (17f * scale) + off),
                new PointF((12f * scale) + off, (22f * scale) + off),
                new PointF((22f * scale) + off, (17f * scale) + off)
            };
            using (Pen botPen = new Pen(Color.FromArgb(255, 161, 161, 170), Math.Max(1.2f, scale * 1.8f)))
            {
                botPen.StartCap = LineCap.Round;
                botPen.EndCap = LineCap.Round;
                g.DrawLines(botPen, botPts);
            }

            bmp.Save(outPath, ImageFormat.Png);
        }
    }

    public static void GenerateIco(string png32Path, string icoPath)
    {
        using (Bitmap bmp = new Bitmap(png32Path))
        {
            IntPtr hIcon = bmp.GetHicon();
            using (Icon icon = Icon.FromHandle(hIcon))
            using (FileStream fs = new FileStream(icoPath, FileMode.Create))
            {
                icon.Save(fs);
            }
        }
    }
}
"@

Add-Type -TypeDefinition $csharpSource -ReferencedAssemblies System.Drawing

[GlastorIconGenerator]::GeneratePng(16, "public\favicon\favicon-16x16.png")
[GlastorIconGenerator]::GeneratePng(32, "public\favicon\favicon-32x32.png")
[GlastorIconGenerator]::GeneratePng(180, "public\favicon\apple-touch-icon.png")
[GlastorIconGenerator]::GeneratePng(192, "public\favicon\android-chrome-192x192.png")
[GlastorIconGenerator]::GeneratePng(512, "public\favicon\android-chrome-512x512.png")
[GlastorIconGenerator]::GenerateIco("public\favicon\favicon-32x32.png", "public\favicon\favicon.ico")

Write-Output "All PNG and ICO favicons generated cleanly!"

