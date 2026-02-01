package com.xenoamess.untar;

import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveOutputStream;
import org.apache.commons.compress.archivers.zip.ZipArchiveEntry;
import org.apache.commons.compress.archivers.zip.ZipArchiveOutputStream;
import org.apache.commons.compress.compressors.gzip.GzipCompressorOutputStream;
import org.apache.commons.compress.compressors.xz.XZCompressorOutputStream;
import org.apache.commons.compress.compressors.bzip2.BZip2CompressorOutputStream;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.*;

class MainTest {

    @TempDir
    Path tempDir;

    private Path archiveDir;
    private Path extractDir;

    @BeforeEach
    void setUp() throws IOException {
        archiveDir = Files.createDirectory(tempDir.resolve("archives"));
        extractDir = Files.createDirectory(tempDir.resolve("extract"));
    }

    @Test
    void testExtractTar() throws Exception {
        Path tarFile = archiveDir.resolve("test.tar");
        createTarArchive(tarFile, "test.txt", "Hello World");

        Main main = new Main();
        main.directory = extractDir.toString();
        main.archiveFile = tarFile.toString();
        main.verbose = false;

        int result = main.call();
        assertEquals(0, result);

        Path extractedFile = extractDir.resolve("test.txt");
        assertTrue(Files.exists(extractedFile));
        assertEquals("Hello World", Files.readString(extractedFile));
    }

    @Test
    void testExtractTarGz() throws Exception {
        Path tarFile = archiveDir.resolve("test.tar.gz");
        createTarGzArchive(tarFile, "test.txt", "Hello Gzip");

        Main main = new Main();
        main.directory = extractDir.toString();
        main.archiveFile = tarFile.toString();
        main.verbose = false;

        int result = main.call();
        assertEquals(0, result);

        Path extractedFile = extractDir.resolve("test.txt");
        assertTrue(Files.exists(extractedFile));
        assertEquals("Hello Gzip", Files.readString(extractedFile));
    }

    @Test
    void testExtractTarBz2() throws Exception {
        Path tarFile = archiveDir.resolve("test.tar.bz2");
        createTarBz2Archive(tarFile, "test.txt", "Hello Bzip2");

        Main main = new Main();
        main.directory = extractDir.toString();
        main.archiveFile = tarFile.toString();
        main.verbose = false;

        int result = main.call();
        assertEquals(0, result);

        Path extractedFile = extractDir.resolve("test.txt");
        assertTrue(Files.exists(extractedFile));
        assertEquals("Hello Bzip2", Files.readString(extractedFile));
    }

    @Test
    void testExtractTarXz() throws Exception {
        Path tarFile = archiveDir.resolve("test.tar.xz");
        createTarXzArchive(tarFile, "test.txt", "Hello Xz");

        Main main = new Main();
        main.directory = extractDir.toString();
        main.archiveFile = tarFile.toString();
        main.verbose = false;

        int result = main.call();
        assertEquals(0, result);

        Path extractedFile = extractDir.resolve("test.txt");
        assertTrue(Files.exists(extractedFile));
        assertEquals("Hello Xz", Files.readString(extractedFile));
    }

    @Test
    void testExtractZip() throws Exception {
        Path zipFile = archiveDir.resolve("test.zip");
        createZipArchive(zipFile, "test.txt", "Hello Zip");

        Main main = new Main();
        main.directory = extractDir.toString();
        main.archiveFile = zipFile.toString();
        main.verbose = false;

        int result = main.call();
        assertEquals(0, result);

        Path extractedFile = extractDir.resolve("test.txt");
        assertTrue(Files.exists(extractedFile));
        assertEquals("Hello Zip", Files.readString(extractedFile));
    }

    @Test
    void testExtractTgz() throws Exception {
        Path tgzFile = archiveDir.resolve("test.tgz");
        createTarGzArchive(tgzFile, "test.txt", "Hello Tgz");

        Main main = new Main();
        main.directory = extractDir.toString();
        main.archiveFile = tgzFile.toString();
        main.verbose = false;

        int result = main.call();
        assertEquals(0, result);

        Path extractedFile = extractDir.resolve("test.txt");
        assertTrue(Files.exists(extractedFile));
        assertEquals("Hello Tgz", Files.readString(extractedFile));
    }

    @Test
    void testExtractNonExistentFile() throws Exception {
        Main main = new Main();
        main.directory = extractDir.toString();
        main.archiveFile = "/nonexistent/file.tar";
        main.verbose = false;

        int result = main.call();
        assertEquals(1, result);
    }

    @Test
    void testExtractMultipleFiles() throws Exception {
        Path tarFile = archiveDir.resolve("multi.tar");
        createTarArchive(tarFile,
            "file1.txt", "Content 1",
            "file2.txt", "Content 2",
            "subdir/file3.txt", "Content 3");

        Main main = new Main();
        main.directory = extractDir.toString();
        main.archiveFile = tarFile.toString();
        main.verbose = false;

        int result = main.call();
        assertEquals(0, result);

        assertTrue(Files.exists(extractDir.resolve("file1.txt")));
        assertTrue(Files.exists(extractDir.resolve("file2.txt")));
        assertTrue(Files.exists(extractDir.resolve("subdir/file3.txt")));
        assertEquals("Content 1", Files.readString(extractDir.resolve("file1.txt")));
        assertEquals("Content 2", Files.readString(extractDir.resolve("file2.txt")));
        assertEquals("Content 3", Files.readString(extractDir.resolve("subdir/file3.txt")));
    }

    @Test
    void testExtractToCustomDirectory() throws Exception {
        Path tarFile = archiveDir.resolve("test.tar");
        createTarArchive(tarFile, "test.txt", "Custom Dir Test");

        Path customDir = tempDir.resolve("custom");
        Files.createDirectories(customDir);

        Main main = new Main();
        main.directory = customDir.toString();
        main.archiveFile = tarFile.toString();
        main.verbose = false;

        int result = main.call();
        assertEquals(0, result);

        assertTrue(Files.exists(customDir.resolve("test.txt")));
        assertEquals("Custom Dir Test", Files.readString(customDir.resolve("test.txt")));
    }

    // Helper methods to create test archives

    private void createTarArchive(Path file, String... entries) throws IOException {
        try (TarArchiveOutputStream taos = new TarArchiveOutputStream(new FileOutputStream(file.toFile()))) {
            for (int i = 0; i < entries.length; i += 2) {
                String name = entries[i];
                String content = entries[i + 1];
                byte[] data = content.getBytes(StandardCharsets.UTF_8);

                TarArchiveEntry entry = new TarArchiveEntry(name);
                entry.setSize(data.length);
                taos.putArchiveEntry(entry);
                taos.write(data);
                taos.closeArchiveEntry();
            }
        }
    }

    private void createTarGzArchive(Path file, String entryName, String content) throws IOException {
        try (TarArchiveOutputStream taos = new TarArchiveOutputStream(
                new GzipCompressorOutputStream(new FileOutputStream(file.toFile())))) {
            byte[] data = content.getBytes(StandardCharsets.UTF_8);
            TarArchiveEntry entry = new TarArchiveEntry(entryName);
            entry.setSize(data.length);
            taos.putArchiveEntry(entry);
            taos.write(data);
            taos.closeArchiveEntry();
        }
    }

    private void createTarBz2Archive(Path file, String entryName, String content) throws IOException {
        try (TarArchiveOutputStream taos = new TarArchiveOutputStream(
                new BZip2CompressorOutputStream(new FileOutputStream(file.toFile())))) {
            byte[] data = content.getBytes(StandardCharsets.UTF_8);
            TarArchiveEntry entry = new TarArchiveEntry(entryName);
            entry.setSize(data.length);
            taos.putArchiveEntry(entry);
            taos.write(data);
            taos.closeArchiveEntry();
        }
    }

    private void createTarXzArchive(Path file, String entryName, String content) throws IOException {
        try (TarArchiveOutputStream taos = new TarArchiveOutputStream(
                new XZCompressorOutputStream(new FileOutputStream(file.toFile())))) {
            byte[] data = content.getBytes(StandardCharsets.UTF_8);
            TarArchiveEntry entry = new TarArchiveEntry(entryName);
            entry.setSize(data.length);
            taos.putArchiveEntry(entry);
            taos.write(data);
            taos.closeArchiveEntry();
        }
    }

    private void createZipArchive(Path file, String entryName, String content) throws IOException {
        try (ZipArchiveOutputStream zaos = new ZipArchiveOutputStream(new FileOutputStream(file.toFile()))) {
            byte[] data = content.getBytes(StandardCharsets.UTF_8);
            ZipArchiveEntry entry = new ZipArchiveEntry(entryName);
            entry.setSize(data.length);
            zaos.putArchiveEntry(entry);
            zaos.write(data);
            zaos.closeArchiveEntry();
        }
    }
}
