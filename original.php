<?php
declare(strict_types=1);

namespace App\Libraries;

use TCPDF;

/**
 * Custom PDF Library for Shipping Labels.
 *
 * Extends TCPDF to create shipping labels with dynamic table rendering, QR code generation, multiple label support, and cut guidelines.
 */
class TcPdfLib
{
    /**
     * @var TCPDF
     */
    private TCPDF $pdf;

    /**
     * @var array<string, mixed> Configuration settings
     */
    private array $config;

    /**
     * Default page dimensions (100x150mm)
     */
    private const PAGE_WIDTH = 100;
    private const PAGE_HEIGHT = 150;

    /**
     * Default table dimensions
     */
    private const TABLE_WIDTH = 96;
    private const TABLE_HEIGHT = 70.5;
    private const TABLE_GAP = 4.5;

    /**
     * Default margins and spacing
     */
    private const MARGIN_TOP = 2;
    private const MARGIN_SIDE = 2;

    /**
     * Font settings
     */
    private const DEFAULT_FONT = 'helvetica';
    private const DEFAULT_FONT_SIZE = 8;
    private const MIN_FONT_SIZE = 5;

    /**
     * QR code settings
     */
    private const QR_SIZE_RATIO = 0.8;
    private const QR_BORDER = 2;

    /**
     * Row height percentages
     */
    private const ROW_HEIGHT_PERCENTS = [0.4, 0.5, 0.1];

    /**
     * Default configuration
     */
    private const DEFAULT_CONFIG = [
        'page' => [
            'width' => self::PAGE_WIDTH,
            'height' => self::PAGE_HEIGHT,
            'orientation' => 'P',
            'unit' => 'mm',
        ],
        'table' => [
            'width' => self::TABLE_WIDTH,
            'height' => self::TABLE_HEIGHT,
            'gap' => self::TABLE_GAP,
            'margin_top' => self::MARGIN_TOP,
            'margin_side' => self::MARGIN_SIDE,
            'header_col1_width' => 18, // Width for "Penerima:" column
            'margin_divider' => 2, // Divider for X positioning: (page_width - table_width) / margin_divider
        ],
        'font' => [
            'default' => self::DEFAULT_FONT,
            'default_size' => self::DEFAULT_FONT_SIZE,
            'min_size' => self::MIN_FONT_SIZE,
            'brand_font' => 'times',
            'brand_size' => 11,
        ],
        'qr' => [
            'size_ratio' => self::QR_SIZE_RATIO,
            'border' => self::QR_BORDER,
        ],
        'row_heights' => self::ROW_HEIGHT_PERCENTS,
        'debug' => false,
    ];

    /**
     * Constructor
     *
     * @param array<string, mixed> $options Configuration options
     * @throws \RuntimeException If TCPDF initialization fails
     */
    public function __construct(array $options = [])
    {
        try {
            $this->config = $this->mergeConfig($options);
            $this->initializePdf();
            $this->logDebug('TcLibPdf initialized successfully');
        } catch (\Exception $e) {
            $this->logDebug('TcLibPdf initialization failed: ' . $e->getMessage());
            throw new \RuntimeException('Failed to initialize TCPDF: ' . $e->getMessage(), 0, $e);
        }
    }

    /**
     * Merge configuration with defaults.
     *
     * @param array<string, mixed> $options
     * @return array<string, mixed>
     */
    private function mergeConfig(array $options): array
    {
        $config = self::DEFAULT_CONFIG;

        if (isset($options['config'])) {
            if (!is_array($options['config'])) {
                throw new \InvalidArgumentException('Config options must be an array');
            }
            $config = array_replace_recursive($config, $options['config']);
        }

        return $config;
    }

    /**
     * Initialize PDF settings
     *
     * @return void
     */
    private function initializePdf(): void
    {
        $pageConfig = $this->config['page'];

        $this->pdf = new TCPDF(
            $pageConfig['orientation'],
            $pageConfig['unit'],
            [$pageConfig['width'], $pageConfig['height']],
            true,
            'UTF-8',
            false,
            false
        );

        // Prevent automatic page breaks
        $this->pdf->SetAutoPageBreak(false);

        // Set margins to minimum
        $this->pdf->SetMargins(0, 0, 0);
        $this->pdf->SetHeaderMargin(0);
        $this->pdf->SetFooterMargin(0);

        // Remove default header/footer
        $this->pdf->setPrintHeader(false);
        $this->pdf->setPrintFooter(false);
    }

    /**
     * Create a new page with custom dimensions
     *
     * @return $this
     */
    public function createPage(): self
    {
        $this->pdf->AddPage();
        $this->pdf->SetXY(0, 0);
        $this->logDebug('New page created');
        return $this;
    }

    /**
     * Add QR code with custom content
     *
     * @param string $content QR code content
     * @param float $x X position
     * @param float $y Y position
     * @param float $w Width
     * @param float $h Height
     * @return $this
     * @throws \InvalidArgumentException
     */
    public function addQrCode(string $content, float $x = 10, float $y = 10, float $w = 30, float $h = 30): self
    {
        if (empty($content)) {
            throw new \InvalidArgumentException('QR code content cannot be empty');
        }

        $qrConfig = $this->config['qr'];

        // Add QR code with high error correction
        $this->pdf->write2DBarcode($content, 'QRCODE,H', $x, $y, $w, $h, [
            'border' => $qrConfig['border'],
            'vpadding' => 'auto',
            'hpadding' => 'auto',
            'fgcolor' => [0, 0, 0],
            'bgcolor' => false,
            'module_width' => 1,
            'module_height' => 1
        ], 'N');

        $this->logDebug("QR code added at ({$x}, {$y}) with size {$w}x{$h}");
        return $this;
    }

    /**
     * Add table with shipping/package information
     *
     * @param array<int, array<int, string|array<int, string>>>|null $data Table data array
     * @param float|null $y Y position for the table
     * @return $this
     * @throws \InvalidArgumentException If data structure is invalid
     */
    public function addTable(?array $data = null, ?float $y = null): self
    {
        $tableConfig = $this->config['table'];
        $x = ($this->config['page']['width'] - $tableConfig['width']) / $tableConfig['margin_divider'];
        $y = $y ?? $tableConfig['margin_top'];
        $rowHeights = $this->calculateRowHeights();

        // Default data if none provided
        if ($data === null) {
            $data = $this->getDefaultData();
        }

        // Validate data structure
        $this->validateTableData($data);

        $fontConfig = $this->config['font'];
        $this->pdf->SetFont($fontConfig['default'], '', $fontConfig['default_size']);
        $this->pdf->SetXY($x, $y);
        $this->drawTableWithSingleBorders($x, $y, $data, $tableConfig['width'], $rowHeights);

        $this->logDebug("Table added at ({$x}, {$y}) with " . count($data) . " rows");
        return $this;
    }

    /**
     * Add multiple labels to the PDF
     *
     * @param array<int, array<int, array<int, string|array<int, string>>>> $labelsData Array of label data arrays
     * @return $this
     * @throws \InvalidArgumentException If labels data is invalid
     */
    public function addMultipleLabels(array $labelsData): self
    {
        if (empty($labelsData)) {
            throw new \InvalidArgumentException('Labels data must be a non-empty array');
        }

        $maxLabelsPerPage = 2; // Each page fits exactly 2 labels
        $tableConfig = $this->config['table'];
        $labelCount = count($labelsData); // Cache count for performance

        $this->logDebug("Adding {$labelCount} labels");

        for ($index = 0; $index < $labelCount; $index++) {
            $data = $labelsData[$index];
            $labelsOnCurrentPage = $index % $maxLabelsPerPage;
            if ($labelsOnCurrentPage == 0 && $index > 0) {
                $this->pdf->AddPage();
                $this->logDebug("New page added for label " . ($index + 1));
            }

            $y = $tableConfig['margin_top'] + ($labelsOnCurrentPage * ($tableConfig['height'] + $tableConfig['gap']));
            $this->addTable($data, $y);

            // Only check once per loop, not every time
            if ($index < $labelCount - 1 && $labelsOnCurrentPage < $maxLabelsPerPage - 1) {
                $this->addCutGuideline($y + $tableConfig['height'] + 2);
            }
        }

        $this->logDebug("All labels added successfully");
        return $this;
    }

    /**
     * Draw dashed cut guideline between tables
     *
     * @param float $y Y position (default: 75)
     * @return $this
     */
    public function addCutGuideline(float $y = 75): self
    {
        $x = 0;
        $width = $this->config['page']['width'];

        $this->pdf->SetLineWidth(0.3);
        $this->pdf->SetLineStyle(['dash' => 2, 'gap' => 2]);
        $this->pdf->Line($x, $y, $x + $width, $y);
        $this->pdf->SetLineWidth(0.2);
        $this->pdf->SetLineStyle(['dash' => 0, 'gap' => 0]);

        $this->logDebug("Cut guideline added at y={$y}");
        return $this;
    }

    /**
     * Calculate row heights based on percentages
     *
     * @return array<int, float>
     */
    private function calculateRowHeights(): array
    {
        $tableConfig = $this->config['table'];
        return array_map(function ($percent) use ($tableConfig) {
            return $percent * $tableConfig['height'];
        }, $this->config['row_heights']);
    }

    /**
     * Get default table data.
     *
     * @return array<int, array<int, string|array<int, string>>> Default data for the table.
     */
    private function getDefaultData(): array
    {
        return [
            ['Aswanto Iwan', 'Jalan Pelita IV No. 92, RT 08 RW 06, Kelurahan Sei Timur, Kecamatan Medan Timur, Medan, Sumatera Utara, 79831', '08267398xxxx'],
            ['qr', ['Andalas Branded', 'TikTok: @andalasbranded', 'IG: andalasbranded', 'WhatsApp: 08267398xxxx']],
            ['Dimensions: 30x20x15 cm', '1202025']
        ];
    }

    /**
     * Validate table data structure.
     *
     * @param array<int, array<int, string|array<int, string>>> $data
     */
    private function validateTableData(array $data): void
    {
        foreach ($data as $rowIndex => $row) {
            if (!is_array($row)) {
                throw new \InvalidArgumentException("Row {$rowIndex} must be an array");
            }
        }
    }

    /**
     * Draw table with single borders
     *
     * @param float $x X position
     * @param float $y Y position
     * @param array<int, array<int, string|array<int, string>>> $data Table data
     * @param float $width Table width
     * @param array<int, float> $rowHeights Row heights array
     * @return $this
     */
    private function drawTableWithSingleBorders(float $x, float $y, array $data, float $width, array $rowHeights): self
    {
        $totalHeight = array_sum($rowHeights);

        if (!$this->isTableFittingOnPage($y, $totalHeight)) {
            $this->logDebug("Table skipped - would exceed page height");
            return $this;
        }

        $this->pdf->SetLineWidth(0.2);
        $this->pdf->Rect($x, $y, $width, $totalHeight);

        $currentY = $y;

        for ($rowIndex = 0; $rowIndex < count($data); $rowIndex++) {
            $row = $data[$rowIndex];
            $height = $rowHeights[$rowIndex];

            if ($rowIndex > 0) {
                $this->pdf->Line($x, $currentY, $x + $width, $currentY);
            }

            $this->renderTableRow($x, $currentY, $row, $width, $height, $rowIndex);
            $this->drawVerticalLines($x, $currentY, $row, $width, $height, $rowIndex);

            $currentY += $height;
        }

        return $this;
    }

    /**
     * Check if table fits on page
     *
     * @param float $y Y position
     * @param float $totalHeight Total height of the table
     * @return bool
     */
    private function isTableFittingOnPage(float $y, float $totalHeight): bool
    {
        return ($y + $totalHeight <= $this->config['page']['height']);
    }

    /**
     * Render a single table row
     *
     * @param float $x X position
     * @param float $y Y position
     * @param array<int, string|array<int, string>> $row Row data
     * @param float $width Table width
     * @param float $height Row height
     * @param int $rowIndex Row index
     */
    private function renderTableRow(float $x, float $y, array $row, float $width, float $height, int $rowIndex): void
    {
        $tableConfig = $this->config['table'];
        $this->pdf->SetXY($x + $tableConfig['margin_side'], $y + $tableConfig['margin_side']);

        if ($rowIndex == 0) {
            $this->renderHeaderRow($x, $y, $row, $width, $height);
        } else {
            $this->renderDataRow($x, $y, $row, $width, $height, $rowIndex);
        }
    }

    /**
     * Render header row (first row)
     *
     * @param float $x X position
     * @param float $y Y position
     * @param array<int, string|array<int, string>> $row Row data
     * @param float $width Table width
     * @param float $height Row height
     */
    private function renderHeaderRow(float $x, float $y, array $row, float $width, float $height): void
    {
        if (count($row) > 1) {
            $this->renderMultiColumnHeader($x, $y, $row, $width, $height);
        } else {
            $this->renderSingleColumnHeader($x, $y, $row, $width, $height);
        }
    }

    /**
     * Render multi-column header
     *
     * @param float $x X position
     * @param float $y Y position
     * @param array<int, string|array<int, string>> $row Row data
     * @param float $width Table width
     * @param float $height Row height
     */
    private function renderMultiColumnHeader(float $x, float $y, array $row, float $width, float $height): void
    {
        $tableConfig = $this->config['table'];
        $col1Width = $tableConfig['header_col1_width'];
        $col2Width = $width - $col1Width;
        $fontConfig = $this->config['font'];

        // Only set font if it changes
        static $lastFont = null;
        $desiredFont = $fontConfig['default'] . '-' . $fontConfig['default_size'];
        if ($lastFont !== $desiredFont) {
            $this->pdf->SetFont($fontConfig['default'], '', $fontConfig['default_size']);
            $lastFont = $desiredFont;
        }
        $this->pdf->writeHTMLCell($col1Width - 2, $height - 4, $x + 2, $y + 2, 'Penerima:', 0, 1, 0, true, 'L', true);

        $infoSections = $row;
        $styledSections = [];
        foreach ($infoSections as $idx => $section) {
            $escapedSection = htmlspecialchars((string)$section);
            if ($idx === 0 || $idx === 2) {
                $styledSections[] = '<b>' . $escapedSection . '</b>';
            } else {
                $styledSections[] = $escapedSection;
            }
        }
        $cellHtml = implode('<br>', $styledSections);
        $this->pdf->writeHTMLCell($col2Width - 2, $height - 4, $x + 2 + $col1Width, $y + 2, $cellHtml, 0, 1, 0, true, 'L', true);
    }

    /**
     * Render single column header
     *
     * @param float $x X position
     * @param float $y Y position
     * @param array<int, string|array<int, string>> $row Row data
     * @param float $width Table width
     * @param float $height Row height
     */
    private function renderSingleColumnHeader(float $x, float $y, array $row, float $width, float $height): void
    {
        $fontConfig = $this->config['font'];
        $fontSize = $fontConfig['default_size'];
        $minFontSize = $fontConfig['min_size'];
        $cellText = (string)$row[0];

        $fontSize = $this->calculateOptimalFontSize($width - 4, $cellText, $height - 4, $fontSize, $minFontSize);
        $this->pdf->SetFont($fontConfig['default'], '', $fontSize);
        $this->pdf->writeHTMLCell($width - 4, $height - 4, $x + 2, $y + 2, htmlspecialchars($cellText), 0, 1, 0, true, 'L', true);
    }

    /**
     * Render data row
     *
     * @param float $x X position
     * @param float $y Y position
     * @param array<int, string|array<int, string>> $row Row data
     * @param float $width Table width
     * @param float $height Row height
     * @param int $rowIndex Row index
     */
    private function renderDataRow(float $x, float $y, array $row, float $width, float $height, int $rowIndex): void
    {
        $colCount = count($row);

        if ($colCount == 2) {
            if ($rowIndex == 1) {
                $this->renderQrRow($x, $y, $row, $width, $height);
            } else {
                $this->renderTwoColumnRow($x, $y, $row, $width, $height);
            }
        } else {
            $this->renderMultiColumnRow($x, $y, $row, $width, $height, $rowIndex);
        }
    }

    /**
     * Render QR code row (row 2)
     *
     * @param float $x X position
     * @param float $y Y position
     * @param array<int, string|array<int, string>> $row Row data
     * @param float $width Table width
     * @param float $height Row height
     */
    private function renderQrRow(float $x, float $y, array $row, float $width, float $height): void
    {
        // Calculate column widths and extract QR config
        $col1Width = $width / 2; // First column takes half the width
        $col2Width = $width / 2; // Second column takes the other half
        $qrConfig = $this->config['qr'];

        // First column: QR code
        // Calculate QR code dimensions and position
        $qrSize = min($col1Width, $height) * $qrConfig['size_ratio']; // QR code size is based on the smaller of column width and row height
        $qrX = $x + ($col1Width - $qrSize) / 2; // Center the QR code horizontally in the first column
        $qrY = $y + ($height - $qrSize) / 2; // Center the QR code vertically in the row
        $qrContent = (string)$row[0]; // QR code content is in the first element of the row array
        $this->addQrCode($qrContent, $qrX, $qrY, $qrSize, $qrSize); // Add the QR code to the PDF

        // Second column: HTML content
        $this->renderQrRowContent($x, $y, $row, $col1Width, $col2Width, $height);
    }

    /**
     * Render QR row content
     *
     * @param float $x X position
     * @param float $y Y position
     * @param array<int, string|array<int, string>> $row Row data
     * @param float $col1Width First column width
     * @param float $col2Width Second column width
     * @param float $height Row height
     */
    private function renderQrRowContent(float $x, float $y, array $row, float $col1Width, float $col2Width, float $height): void
    {
        $fontConfig = $this->config['font'];
        $fontSize = 7;
        $lines = is_array($row[1]) ? $row[1] : [(string)$row[1]];
        $lineHeights = [];
        $contentHeight = 0;

        // Cache font state to avoid redundant SetFont calls
        static $lastFont = null;

        foreach ($lines as $idx => $line) {
            $desiredFont = ($idx === 0)
                ? $fontConfig['brand_font'] . '-B-' . $fontConfig['brand_size']
                : $fontConfig['default'] . '-' . $fontSize;
            if ($lastFont !== $desiredFont) {
                if ($idx === 0) {
                    $this->pdf->SetFont($fontConfig['brand_font'], 'B', $fontConfig['brand_size']);
                } else {
                    $this->pdf->SetFont($fontConfig['default'], '', $fontSize);
                }
                $lastFont = $desiredFont;
            }
            $lineHeight = $this->pdf->getStringHeight($col2Width, (string)$line, false, true, '', 1, 0, 0, 12);
            $lineHeights[] = $lineHeight;
            $contentHeight += $lineHeight;
        }

        $cellY = $y + (($height - $contentHeight) / 2);
        $currentLineY = $cellY;

        foreach ($lines as $idx => $line) {
            $desiredFont = ($idx === 0)
                ? $fontConfig['brand_font'] . '-B-' . $fontConfig['brand_size']
                : $fontConfig['default'] . '-' . $fontSize;
            if ($lastFont !== $desiredFont) {
                if ($idx === 0) {
                    $this->pdf->SetFont($fontConfig['brand_font'], 'B', $fontConfig['brand_size']);
                } else {
                    $this->pdf->SetFont($fontConfig['default'], '', $fontSize);
                }
                $lastFont = $desiredFont;
            }
            $this->pdf->writeHTMLCell($col2Width, $lineHeights[$idx], $x + $col1Width, $currentLineY, (string)$line, 0, 1, 0, true, 'C', true);
            $currentLineY += $lineHeights[$idx];
        }
    }

    /**
     * Render two column row
     *
     * @param float $x X position
     * @param float $y Y position
     * @param array<int, string|array<int, string>> $row Row data
     * @param float $width Table width
     * @param float $height Row height
     */
    private function renderTwoColumnRow(float $x, float $y, array $row, float $width, float $height): void
    {
        // Define column widths and extract font config
        $col1Width = 25; // Fixed width for the first column
        $col2Width = $width - $col1Width; // Remaining width for the second column
        $fontConfig = $this->config['font'];

        // First column
        $fontSize = $this->calculateOptimalFontSize($col1Width - 2, (string)$row[0], $height - 4, 7, $fontConfig['min_size']);
        $this->pdf->SetFont($fontConfig['default'], '', $fontSize);
        $this->pdf->writeHTMLCell($col1Width - 2, $height - 4, $x + 2, $y + 2, htmlspecialchars((string)$row[0]), 0, 1, 0, true, 'L', true);

        // Second column
        $fontSize = $this->calculateOptimalFontSize($col2Width - 2, (string)$row[1], $height - 4, 7, $fontConfig['min_size']);
        $this->pdf->SetFont($fontConfig['default'], '', $fontSize);
        $this->pdf->writeHTMLCell($col2Width - 2, $height - 4, $x + 2 + $col1Width, $y + 2, htmlspecialchars((string)$row[1]), 0, 1, 0, true, 'R', true);
    }

    /**
     * Render multi-column row
     *
     * @param float $x X position
     * @param float $y Y position
     * @param array<int, string|array<int, string>> $row Row data
     * @param float $width Table width
     * @param float $height Row height
     * @param int $rowIndex Row index
     */
    private function renderMultiColumnRow(float $x, float $y, array $row, float $width, float $height, int $rowIndex): void
    {
        // Calculate column width and initialize current X position
        $colWidth = ($width - 4) / count($row); // Divide the available width equally among the columns
        $currentX = $x + 2; // Start position for the first column
        $fontConfig = $this->config['font'];

        if ($rowIndex == 2 && count($row) == 2) {
            // Row 3, 2 columns: use HTML for left/right alignment
            $this->pdf->SetFont($fontConfig['default'], '', 7);
            $this->pdf->writeHTMLCell($colWidth, $height - 4, $currentX, $y + 2, htmlspecialchars((string)$row[0]), 0, 1, 0, true, 'L', true);
            $currentX += $colWidth;
            $this->pdf->writeHTMLCell($colWidth, $height - 4, $currentX, $y + 2, htmlspecialchars((string)$row[1]), 0, 1, 0, true, 'R', true);
        } else {
            for ($j = 0; $j < count($row); $j++) {
                $cellText = (string)$row[$j];
                $fontSize = $this->calculateOptimalFontSize($colWidth, $cellText, $height - 4, 7, $fontConfig['min_size']);
                $this->pdf->SetFont($fontConfig['default'], '', $fontSize);
                $this->pdf->writeHTMLCell($colWidth, $height - 4, $currentX, $y + 2, htmlspecialchars($cellText), 0, 1, 0, true, 'L', true);
                $currentX += $colWidth;
            }
        }
    }

    /**
     * Calculate optimal font size for text to fit in cell
     *
     * @param float $width Cell width
     * @param string $text Text content
     * @param float $height Cell height
     * @param int $startFontSize Starting font size
     * @param int $minFontSize Minimum font size
     * @return int Optimal font size
     */
    private function calculateOptimalFontSize(float $width, string $text, float $height, int $startFontSize, int $minFontSize): int
    {
        $fontSize = $startFontSize;
        $min = $minFontSize;
        $max = $startFontSize;
        // Extract font configuration
        $fontConfig = $this->config['font'];

        while ($min <= $max) {
            $mid = (int)(($min + $max) / 2);
            $this->pdf->SetFont($fontConfig['default'], '', $mid);
            $cellHeight = $this->pdf->getStringHeight($width, $text);

            if ($cellHeight > $height) {
                $max = $mid - 1;
            } else {
                $fontSize = $mid;
                $min = $mid + 1;
            }
        }

        return $fontSize;
    }

    /**
     * Draw vertical lines for table row
     *
     * @param float $x X position
     * @param float $y Y position
     * @param array<int, string|array<int, string>> $row Row data
     * @param float $width Table width
     * @param float $height Row height
     * @param int $rowIndex Row index
     */
    private function drawVerticalLines(float $x, float $y, array $row, float $width, float $height, int $rowIndex): void
    {
        // Draw vertical lines for this row, except for row 1 (header, i==0) and row 3 (i==2)
        // Vertical lines are drawn for rows that are not the header (rowIndex 0) or the third row (rowIndex 2),
        // and only if there is more than one column.
        if ($rowIndex !== 0 && $rowIndex !== 2) {
            if (count($row) > 1) {
                $colWidths = $this->calculateColumnWidths($row, $width, $rowIndex);

                // Draw vertical lines based on calculated column widths
                $currentLineX = $x;
                foreach ($colWidths as $colWidth) {
                    $currentLineX += $colWidth;
                    $this->pdf->Line($currentLineX, $y, $currentLineX, $y + $height);
                }
            }
        }
    }

    /**
     * Calculate column widths for table row
     *
     * @param array<int, string|array<int, string>> $row Row data
     * @param float $width Table width
     * @param int $rowIndex Row index
     * @return array<int, float>
     */
    private function calculateColumnWidths(array $row, float $width, int $rowIndex): array
    {
        $colCount = count($row);
        $colWidths = [];

        if ($colCount === 2) {
            // Special handling for two-column rows
            if ($rowIndex === 1) { // Row 2 (QR code row) has a 50/50 split
                $colWidths[] = $width / 2;
            } else { // Other two-column rows have a fixed first column width
                $colWidths[] = 25;
            }
        } else {
            // For rows with more than two columns, distribute width equally
            $equalColWidth = $width / $colCount;
            for ($j = 0; $j < $colCount - 1; $j++) {
                $colWidths[] = $equalColWidth;
            }
        }

        return $colWidths;
    }

    /**
     * Log debug information using CodeIgniter's logger.
     *
     * @param string $message Debug message
     * @return void
     */
    private function logDebug(string $message): void
    {
        if (!empty($this->config['debug'])) {
            if (!function_exists('service')) {
                return; // Fallback: do nothing if not in CI context
            }
            $logger = service('logger');
            if ($logger) {
                $logger->debug('[TcPdfLib] ' . $message);
            }
        }
    }

    /**
     * Get current configuration.
     *
     * @return array<string, mixed> Configuration.
     */
    public function getConfig(): array
    {
        return $this->config;
    }

    /**
     * Update configuration.
     *
     * @param array<string, mixed> $config New configuration.
     * @return $this
     */
    public function setConfig(array $config): self
    {
        $this->config = array_replace_recursive($this->config, $config);
        return $this;
    }

    /**
     * Output the PDF to browser or file
     *
     * @param string $filename Output filename
     * @param string $dest Destination (I=inline, D=download, F=file, S=string)
     * @return void|string
     */
    public function output(string $filename = 'document.pdf', string $dest = 'I')
    {
        $this->logDebug("PDF output: {$filename} ({$dest})");
        return $this->pdf->Output($filename, $dest);
    }

    /**
     * Get the underlying TCPDF object for advanced usage
     *
     * @return TCPDF
     */
    public function getPdf(): TCPDF
    {
        return $this->pdf;
    }
}