package com.xenoamess.untar;

import picocli.CommandLine;
import picocli.CommandLine.Command;
import picocli.CommandLine.Option;
import picocli.CommandLine.Parameters;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Locale;
import java.util.ResourceBundle;
import java.util.concurrent.Callable;
import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream;
import org.apache.commons.compress.archivers.zip.ZipArchiveEntry;
import org.apache.commons.compress.archivers.zip.ZipArchiveInputStream;
import org.apache.commons.compress.compressors.gzip.GzipCompressorInputStream;
import org.apache.commons.compress.compressors.xz.XZCompressorInputStream;
import org.apache.commons.compress.compressors.bzip2.BZip2CompressorInputStream;

@Command(name = "untar", description = "%n%@Extract tar/tar.gz/tgz/tar.xz/tar.bz2/tar.zip packages")
public class Main implements Callable<Integer> {

    private static final ResourceBundle MESSAGES;

    static {
        Locale locale = Locale.getDefault();
        if (locale.getLanguage().equals("zh")) {
            MESSAGES = ResourceBundle.getBundle("Messages", Locale.CHINA);
        } else {
            MESSAGES = ResourceBundle.getBundle("Messages", Locale.ENGLISH);
        }
    }

    private String msg(String key) {
        return MESSAGES.getString(key);
    }

    @Option(names = {"-C", "--directory"}, descriptionKey = "directory", paramLabel = "DIR")
    String directory = ".";

    @Option(names = {"-v", "--verbose"}, descriptionKey = "verbose")
    boolean verbose = true;

    @Option(names = {"-h", "--help"}, usageHelp = true, descriptionKey = "help")
    boolean help = false;

    @Parameters(paramLabel = "FILE", descriptionKey = "file")
    String archiveFile;

    @Override
    public Integer call() throws Exception {
        File file = new File(archiveFile);
        if (!file.exists()) {
            System.err.println(msg("error.fileNotFound") + ": " + archiveFile);
            return 1;
        }

        Path destDir = Paths.get(directory);
        Files.createDirectories(destDir);

        String fileName = file.getName().toLowerCase();
        try {
            // 优先使用后缀名对应的解压方式
            if (!tryExtractBySuffix(file, destDir, fileName)) {
                // 如果后缀名方式失败，尝试所有支持的格式
                tryExtractAllFormats(file, destDir);
            }
            if (verbose) {
                System.out.println(msg("done") + ": " + file.getName());
            }
        } catch (Exception e) {
            System.err.println(msg("error.extractFailed") + ": " + e.getMessage());
            if (verbose) {
                e.printStackTrace();
            }
            return 1;
        }
        return 0;
    }

    /**
     * 优先使用后缀名对应的解压方式
     * @return true 如果成功解压，false 如果后缀名不匹配或解压失败
     */
    private boolean tryExtractBySuffix(File file, Path destDir, String fileName) throws IOException {
        if (fileName.endsWith(".tar.gz") || fileName.endsWith(".tgz")) {
            extractTarGz(file, destDir);
            return true;
        } else if (fileName.endsWith(".tar.xz")) {
            extractTarXz(file, destDir);
            return true;
        } else if (fileName.endsWith(".tar.bz2")) {
            extractTarBz2(file, destDir);
            return true;
        } else if (fileName.endsWith(".zip")) {
            extractZip(file, destDir);
            return true;
        } else if (fileName.endsWith(".tar")) {
            extractTar(file, destDir);
            return true;
        }
        return false;
    }

    /**
     * 尝试所有支持的压缩格式
     */
    private void tryExtractAllFormats(File file, Path destDir) throws IOException {
        // 按照常见程度排序尝试：zip, tar.gz, tar.bz2, tar.xz, tar
        if (tryExtractZip(file, destDir)) {
            return;
        }
        if (tryExtractTarGz(file, destDir)) {
            return;
        }
        if (tryExtractTarBz2(file, destDir)) {
            return;
        }
        if (tryExtractTarXz(file, destDir)) {
            return;
        }
        if (tryExtractTar(file, destDir)) {
            return;
        }
        throw new IOException(msg("error.unsupportedFormat"));
    }

    private boolean tryExtractZip(File file, Path destDir) {
        try {
            extractZip(file, destDir);
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    private boolean tryExtractTarGz(File file, Path destDir) {
        try {
            extractTarGz(file, destDir);
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    private boolean tryExtractTarBz2(File file, Path destDir) {
        try {
            extractTarBz2(file, destDir);
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    private boolean tryExtractTarXz(File file, Path destDir) {
        try {
            extractTarXz(file, destDir);
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    private boolean tryExtractTar(File file, Path destDir) {
        try {
            extractTar(file, destDir);
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    private void extractTarGz(File file, Path destDir) throws IOException {
        try (FileInputStream fis = new FileInputStream(file);
             GzipCompressorInputStream gzis = new GzipCompressorInputStream(fis);
             TarArchiveInputStream tis = new TarArchiveInputStream(gzis)) {
            extractTar(tis, destDir);
        }
    }

    private void extractTarXz(File file, Path destDir) throws IOException {
        try (FileInputStream fis = new FileInputStream(file);
             XZCompressorInputStream xzis = new XZCompressorInputStream(fis);
             TarArchiveInputStream tis = new TarArchiveInputStream(xzis)) {
            extractTar(tis, destDir);
        }
    }

    private void extractTarBz2(File file, Path destDir) throws IOException {
        try (FileInputStream fis = new FileInputStream(file);
             BZip2CompressorInputStream bzis = new BZip2CompressorInputStream(fis);
             TarArchiveInputStream tis = new TarArchiveInputStream(bzis)) {
            extractTar(tis, destDir);
        }
    }

    private void extractTar(File file, Path destDir) throws IOException {
        try (FileInputStream fis = new FileInputStream(file);
             TarArchiveInputStream tis = new TarArchiveInputStream(fis)) {
            extractTar(tis, destDir);
        }
    }

    private void extractTar(TarArchiveInputStream tis, Path destDir) throws IOException {
        TarArchiveEntry entry;
        while ((entry = tis.getNextTarEntry()) != null) {
            Path entryPath = destDir.resolve(entry.getName());
            if (entry.isDirectory()) {
                Files.createDirectories(entryPath);
            } else {
                Files.createDirectories(entryPath.getParent());
                try (FileOutputStream fos = new FileOutputStream(entryPath.toFile())) {
                    byte[] buffer = new byte[8192];
                    int len;
                    while ((len = tis.read(buffer)) > 0) {
                        fos.write(buffer, 0, len);
                    }
                }
                // 保留文件权限
                if (entry.getMode() > 0) {
                    entryPath.toFile().setExecutable((entry.getMode() & 0100) != 0);
                    entryPath.toFile().setWritable((entry.getMode() & 0200) != 0);
                    entryPath.toFile().setReadable((entry.getMode() & 0400) != 0);
                }
            }
            if (verbose) {
                System.out.println(msg("extracting") + ": " + entry.getName());
            }
        }
    }

    private void extractZip(File file, Path destDir) throws IOException {
        try (FileInputStream fis = new FileInputStream(file);
             ZipArchiveInputStream zis = new ZipArchiveInputStream(fis)) {
            ZipArchiveEntry entry;
            while ((entry = zis.getNextEntry()) != null) {
                Path entryPath = destDir.resolve(entry.getName());
                if (entry.isDirectory()) {
                    Files.createDirectories(entryPath);
                } else {
                    Files.createDirectories(entryPath.getParent());
                    try (FileOutputStream fos = new FileOutputStream(entryPath.toFile())) {
                        byte[] buffer = new byte[8192];
                        int len;
                        while ((len = zis.read(buffer)) > 0) {
                            fos.write(buffer, 0, len);
                        }
                    }
                }
                if (verbose) {
                    System.out.println(msg("extracting") + ": " + entry.getName());
                }
            }
        }
    }

    public static void main(String[] args) {
        int exitCode = new CommandLine(new Main()).execute(args);
        System.exit(exitCode);
    }
}
