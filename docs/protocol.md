# Protokół Gadu-Gadu 6.0

© Copyright 2001-2004 Autorzy

---

## Spis treści

1. [Protokół Gadu-Gadu](#1-protokół-gadu-gadu)
   - 1.1. [Format pakietów i konwencje](#11-format-pakietów-i-konwencje)
   - 1.2. [Zanim się połączymy](#12-zanim-się-połączymy)
   - 1.3. [Logowanie się](#13-logowanie-się)
   - 1.4. [Zmiana stanu](#14-zmiana-stanu)
   - 1.5. [Ludzie przychodzą, ludzie odchodzą](#15-ludzie-przychodzą-ludzie-odchodzą)
   - 1.6. [Wysyłanie wiadomości](#16-wysyłanie-wiadomości)
   - 1.7. [Otrzymywanie wiadomości](#17-otrzymywanie-wiadomości)
   - 1.8. [Ping, pong](#18-ping-pong)
   - 1.9. [Rozłączenie](#19-rozłączenie)
   - 1.10. [Katalog publiczny](#110-katalog-publiczny)
   - 1.11. [Lista kontaktów](#111-lista-kontaktów)
   - 1.12. [Indeks pakietów](#112-indeks-pakietów)
2. [Usługi HTTP](#2-usługi-http)
   - 2.1. [Format danych](#21-format-danych)
   - 2.2. [Tokeny](#22-tokeny)
   - 2.3. [Rejestracja konta](#23-rejestracja-konta)
   - 2.4. [Usunięcie konta](#24-usunięcie-konta)
   - 2.5. [Zmiana hasła](#25-zmiana-hasła)
   - 2.6. [Przypomnienie hasła](#26-przypomnienie-hasła)
3. [Połączenia bezpośrednie](#3-połączenia-bezpośrednie)
   - 3.1. [Nawiązanie połączenia](#31-nawiązanie-połączenia)
   - 3.2. [Przesyłanie plików](#32-przesyłanie-plików)
   - 3.3. [Rozmowy głosowe](#33-rozmowy-głosowe)
4. [Autorzy](#4-autorzy)

---

## Informacje wstępne

Opis protokołu używanego przez Gadu-Gadu bazuje na doświadczeniach przeprowadzonych przez autorów oraz informacjach nadsyłanych przez użytkowników. Żaden klient Gadu-Gadu nie został skrzywdzony podczas badań. Reverse-engineering opiera się głównie na analizie pakietów przesyłanych między klientem a serwerem.

---

## 1. Protokół Gadu-Gadu

### 1.1. Format pakietów i konwencje

Podobnie jak coraz większa ilość komunikatorów, Gadu-Gadu korzysta z protokołu TCP/IP. Każdy pakiet zawiera na początku dwa stałe pola:

```c
struct gg_header {
    int type;    /* typ pakietu */
    int length;  /* długość reszty pakietu */
};
```

Wszystkie zmienne liczbowe są zgodne z kolejnością bajtów maszyn Intela, czyli **Little-Endian**. Wszystkie teksty są kodowane przy użyciu zestawu znaków **CP1250** (windows-1250). Linie kończą się znakami `\r\n`.

Przy opisie struktur, założono, że:
- `char` ma rozmiar 1 bajtu
- `short` ma rozmiar 2 bajtów
- `int` ma rozmiar 4 bajtów
- `long long` ma rozmiar 8 bajtów

Używając architektur innych niż i386, należy zwrócić szczególną uwagę na rozmiar typów zmiennych i kolejność znaków. Poza tym, większość dostępnych obecnie kompilatorów domyślnie wyrównuje zmienne do rozmiaru słowa danej architektury, więc należy wyłączyć tę funkcję.

**GCC:**
```c
struct example {
    // ...
} __attribute__ ((packed));
```

**Microsoft Visual C++:**
```c
#pragma pack(push, 1)
/* deklaracje */
#pragma pack(pop)
```

Pola, których znaczenie jest nieznane, lub nie do końca jasne, oznaczono przedrostkiem `unknown`.

Możliwe jest połączenie za pośrednictwem protokołu TLSv1. Szczegóły znajdują się w poniższym opisie.

---

### 1.2. Zanim się połączymy

Żeby wiedzieć, z jakim serwerem mamy się połączyć, należy za pomocą HTTP połączyć się z `appmsg.gadu-gadu.pl` i wysłać:

```http
GET /appsvc/appmsg4.asp?fmnumber=NUMER&version=WERSJA&fmt=FORMAT&lastmsg=WIADOMOSC
Accept: image/gif, image/jpeg, image/pjpeg, ...
Accept-Language: pl
User-Agent: PRZEGLADARKA
Pragma: no-cache
Host: appmsg.gadu-gadu.pl
```

- **NUMER** - numer Gadu-Gadu
- **WERSJA** - wersja klienta w postaci "A, B, C, D" (np. "5, 0, 5, 107" dla wersji 5.0.5 build 107)
- **FORMAT** - określa czy wiadomość systemowa będzie przesyłana czystym tekstem (brak) czy w HTML (wartość "2")
- **WIADOMOSC** - numer ostatnio otrzymanej wiadomości systemowej

Przykładowe User-Agent:
- `Mozilla/4.04 [en] (Win95; I ;Nav)`
- `Mozilla/4.7 [en] (Win98; I)`
- `Mozilla/4.0 (compatible; MSIE 5.0; Windows NT; DigExt)`
- `Mozilla/4.0 (compatible; MSIE 5.0; Windows 98)`

Na postawione w ten sposób zapytanie, serwer powinien odpowiedzieć:

```
HTTP/1.0 200 OK

0 0 217.17.41.84:8074 217.17.41.84
```

Pierwsze pole jest numerem wiadomości systemowej, a trzecie i czwarte podają nam namiary na właściwy serwer. Jeśli serwer jest niedostępny, zamiast adresu IP jest zwracany tekst `notoperating`. Jeżeli połączenie z portem 8074 nie powiedzie się z jakichś powodów, można się łączyć na port 443.

**Połączenie TLS:** Jeśli klient chce się łączyć za pomocą protokołu TLSv1, wysyła zapytanie do skryptu `appmsg3.asp` i otrzymuje w odpowiedzi adres serwera oraz port 443.

---

### 1.3. Logowanie się

Po połączeniu się portem 8074 lub 443 serwera Gadu-Gadu, otrzymujemy pakiet typu 0x0001:

```c
#define GG_WELCOME 0x0001

struct gg_welcome {
    int seed;    /* klucz szyfrowania hasła */
};
```

Kiedy mamy już tę wartość możemy odesłać pakiet logowania:

```c
#define GG_LOGIN60 0x0015

struct gg_login60 {
    int uin;              /* mój numerek */
    int hash;             /* hash hasła */
    int status;           /* status na dzień dobry */
    int version;          /* moja wersja klienta */
    char unknown1;        /* 0x00 */
    int local_ip;         /* mój adres ip */
    short local_port;     /* port, na którym słucham */
    int external_ip;      /* zewnętrzny adres ip */
    short external_port;  /* zewnętrzny port */
    char image_size;      /* maksymalny rozmiar grafiki w KB */
    char unknown2;        /* 0xbe */
    char description[];   /* opis, nie musi wystąpić */
    int time;             /* czas, nie musi wystąpić */
};
```

Hash hasła można obliczyć następującą funkcją:

```c
int gg_login_hash(char *password, int seed)
{
    unsigned int x, y, z;

    y = seed;

    for (x = 0; *password; password++) {
        x = (x & 0xffffff00) | *password;
        y ^= x;
        y += x;
        x <<= 8;
        y ^= x;
        x <<= 8;
        y -= x;
        x <<= 8;
        y ^= x;

        z = y & 0x1f;
        y = (y << z) | (y >> (32 - z));
    }

    return y;
}
```

**Wersje klientów:**

| Wartość | Wersje klientów |
|---------|-----------------|
| 0x20 | **6.0** |
| 0x1e | 5.7 beta (build 121) |
| 0x1c | 5.7 beta |
| 0x1b | 5.0.5 |
| 0x19 | 5.0.3 |
| 0x18 | 5.0.1, 5.0.0, 4.9.3 |
| 0x17 | 4.9.2 |
| 0x16 | 4.9.1 |
| 0x15 | 4.8.9 |
| 0x14 | 4.8.3, 4.8.1 |
| 0x11 | 4.6.10, 4.6.1 |
| 0x10 | 4.5.22, 4.5.21, 4.5.19, 4.5.17, 4.5.15 |
| 0x0f | 4.5.12 |
| 0x0b | 4.0.30, 4.0.29, 4.0.28, 4.0.25 |

Należy przedstawić się jako co najmniej wersja **6.0**, ponieważ tej wersji protokołu dotyczy poniższy dokument.

Jeśli klient obsługuje rozmowy głosowe, do wersji dodawana jest wartość:

```c
#define GG_HAS_AUDIO_MASK 0x40000000
```

**Odpowiedzi:**

```c
#define GG_LOGIN_OK     0x0003  /* sukces, pakiet o zerowej długości */
#define GG_LOGIN_FAILED 0x0009  /* błąd */
```

---

### 1.4. Zmiana stanu

```c
#define GG_NEW_STATUS 0x0002

struct gg_new_status {
    int status;         /* na jaki zmienić? */
    char description[]; /* opis, nie musi wystąpić */
    int time;           /* czas, nie musi wystąpić */
};
```

**Możliwe stany:**

| Etykieta | Wartość | Znaczenie |
|----------|---------|-----------|
| GG_STATUS_NOT_AVAIL | 0x0001 | Niedostępny |
| GG_STATUS_NOT_AVAIL_DESCR | 0x0015 | Niedostępny (z opisem) |
| GG_STATUS_AVAIL | 0x0002 | Dostępny |
| GG_STATUS_AVAIL_DESCR | 0x0004 | Dostępny (z opisem) |
| GG_STATUS_BUSY | 0x0003 | Zajęty |
| GG_STATUS_BUSY_DESCR | 0x0005 | Zajęty (z opisem) |
| GG_STATUS_INVISIBLE | 0x0014 | Niewidoczny |
| GG_STATUS_INVISIBLE_DESCR | 0x0016 | Niewidoczny z opisem |
| GG_STATUS_BLOCKED | 0x0006 | Zablokowany |
| GG_STATUS_FRIENDS_MASK | 0x8000 | Maska: tylko dla przyjaciół |

Przed rozłączeniem z serwerem należy zmienić stan na `GG_STATUS_NOT_AVAIL` lub `GG_STATUS_NOT_AVAIL_DESCR`.

Maksymalna długość opisu wynosi **70 znaków** plus zero plus 4 bajty na godzinę powrotu = **75 bajtów**.

---

### 1.5. Ludzie przychodzą, ludzie odchodzą

Lista kontaktów jest dzielona na pakiety po **400 wpisów**:

```c
#define GG_NOTIFY_FIRST 0x000f
#define GG_NOTIFY_LAST  0x0010

struct gg_notify {
    int uin;     /* numerek danej osoby */
    char type;   /* rodzaj użytkownika */
};
```

**Typy użytkowników:**

| Etykieta | Wartość | Znaczenie |
|----------|---------|-----------|
| GG_USER_BUDDY | 0x01 | Użytkownik w liście kontaktów |
| GG_USER_FRIEND | 0x02 | Widoczny w trybie "tylko dla przyjaciół" |
| GG_USER_BLOCKED | 0x04 | Zablokowany |

Pusta lista:

```c
#define GG_LIST_EMPTY 0x0012
```

**Odpowiedź serwera:**

```c
#define GG_NOTIFY_REPLY60 0x0011

struct gg_notify_reply60 {
    int uin;               /* numerek plus flagi w najstarszym bajcie */
    char status;           /* status danej osoby */
    int remote_ip;         /* adres IP bezpośrednich połączeń */
    short remote_port;     /* port bezpośrednich połączeń */
    char version;          /* wersja klienta */
    char image_size;       /* maksymalny rozmiar obrazków w KB */
    char unknown1;         /* 0x00 */
    char description_size; /* rozmiar opisu i czasu */
    char description[];    /* opis */
    int time;              /* czas */
};
```

**Flagi w najstarszym bajcie UIN:**

| Etykieta | Wartość | Znaczenie |
|----------|---------|-----------|
| GG_UINFLAG_UNKNOWN1 | 0x10 | Nieznane |
| GG_UINFLAG_UNKNOWN2 | 0x20 | Użytkownik staje się niedostępny |
| GG_UINFLAG_VOICE | 0x40 | Może prowadzić rozmowy głosowe |

**Wartości remote_port:**

| Wartość | Znaczenie |
|---------|-----------|
| 0 | Klient nie obsługuje bezpośrednich połączeń |
| 1 | Klient łączy się zza NAT |
| 2 | Klient nie ma nas w swojej liście |

**Dodawanie/usuwanie kontaktów:**

```c
#define GG_ADD_NOTIFY    0x000d
#define GG_REMOVE_NOTIFY 0x000e

struct gg_add_notify {
    int uin;
    char type;
};
```

**Zmiana stanu kontaktu:**

```c
#define GG_STATUS60 0x000f

struct gg_status60 {
    int uin;            /* numer plus flagi */
    char status;        /* nowy stan */
    int remote_ip;
    short remote_port;
    char version;
    char image_size;
    char unknown1;      /* 0x00 */
    char description[];
    int time;
};
```

---

### 1.6. Wysyłanie wiadomości

```c
#define GG_SEND_MSG 0x000b

struct gg_send_msg {
    int recipient;    /* numer odbiorcy */
    int seq;          /* numer sekwencyjny */
    int class;        /* klasa wiadomości */
    char message[];   /* treść */
};
```

**Klasy wiadomości:**

| Etykieta | Wartość | Znaczenie |
|----------|---------|-----------|
| GG_CLASS_QUEUED | 0x0001 | Wiadomość była zakolejkowana |
| GG_CLASS_MSG | 0x0004 | Osobne okienko |
| GG_CLASS_CHAT | 0x0008 | Istniejące okienko rozmowy |
| GG_CLASS_CTCP | 0x0010 | Dla klienta (nie wyświetlaj) |
| GG_CLASS_ACK | 0x0020 | Bez potwierdzenia |

Maksymalna długość: **2000 znaków** (oryginalny klient: 1989).

**Wiadomości konferencyjne:**

```c
struct gg_msg_recipients {
    char flag;         /* == 1 */
    int count;         /* ilość odbiorców */
    int recipients[];  /* tablica odbiorców */
};
```

**Formatowanie (Rich Text):**

```c
struct gg_msg_richtext {
    char flag;      /* == 2 */
    short length;   /* długość dalszej części */
};

struct gg_msg_richtext_format {
    short position; /* pozycja atrybutu */
    char font;      /* atrybuty czcionki */
    char rgb[3];    /* kolor (opcjonalnie) */
};
```

**Atrybuty czcionki:**

| Etykieta | Wartość | Znaczenie |
|----------|---------|-----------|
| GG_FONT_BOLD | 0x01 | Pogrubiony |
| GG_FONT_ITALIC | 0x02 | Kursywa |
| GG_FONT_UNDERLINE | 0x04 | Podkreślenie |
| GG_FONT_COLOR | 0x08 | Kolor (dodaje rgb[3]) |
| GG_FONT_IMAGE | 0x80 | Obrazek |

**Obrazki:**

```c
struct gg_msg_richtext_image {
    short unknown1;  /* 0x0109 */
    long size;       /* rozmiar */
    long crc32;      /* suma kontrolna */
};

struct gg_msg_image_request {
    char flag;   /* 0x04 */
    int size;
    int crc32;
};

struct gg_msg_image_reply {
    char flag;        /* 0x05 lub 0x06 */
    int size;
    int crc32;
    char filename[];
    char image[];
};
```

**Potwierdzenie:**

```c
#define GG_SEND_MSG_ACK 0x0005

struct gg_send_msg_ack {
    int status;
    int recipient;
    int seq;
};
```

| Etykieta | Wartość | Znaczenie |
|----------|---------|-----------|
| GG_ACK_BLOCKED | 0x0001 | Zablokowano |
| GG_ACK_DELIVERED | 0x0002 | Dostarczono |
| GG_ACK_QUEUED | 0x0003 | Zakolejkowano |
| GG_ACK_MBOXFULL | 0x0004 | Skrzynka pełna |
| GG_ACK_NOT_DELIVERED | 0x0006 | Nie dostarczono |

---

### 1.7. Otrzymywanie wiadomości

```c
#define GG_RECV_MSG 0x000a

struct gg_recv_msg {
    int sender;      /* numer nadawcy */
    int seq;         /* numer sekwencyjny */
    int time;        /* czas nadania (UTC) */
    int class;       /* klasa wiadomości */
    char message[];  /* treść */
};
```

---

### 1.8. Ping, pong

```c
#define GG_PING 0x0008
#define GG_PONG 0x0007
```

Jeśli serwer nie dostanie ping przez **5 minut**, zrywa połączenie.

---

### 1.9. Rozłączenie

```c
#define GG_DISCONNECTING 0x000b
```

Pusty pakiet wysyłany przed rozłączeniem przez serwer (zbyt wiele błędnych haseł lub logowanie z innego miejsca).

---

### 1.10. Katalog publiczny

```c
#define GG_PUBDIR50_REQUEST 0x0014

struct gg_pubdir50 {
    char type;
    int seq;
    char request[];
};

#define GG_PUBDIR50_WRITE  0x01
#define GG_PUBDIR50_READ   0x02
#define GG_PUBDIR50_SEARCH 0x03
```

Parametry w formacie: `nazwa\0wartość\0`

| Etykieta | Wartość | Znaczenie |
|----------|---------|-----------|
| GG_PUBDIR50_UIN | FmNumber | Numer |
| GG_PUBDIR50_FIRSTNAME | firstname | Imię |
| GG_PUBDIR50_LASTNAME | lastname | Nazwisko |
| GG_PUBDIR50_NICKNAME | nickname | Pseudonim |
| GG_PUBDIR50_BIRTHYEAR | birthyear | Rok urodzenia |
| GG_PUBDIR50_CITY | city | Miejscowość |
| GG_PUBDIR50_GENDER | gender | Płeć (1=K, 2=M) |
| GG_PUBDIR50_ACTIVE | ActiveOnly | Tylko dostępni |
| GG_PUBDIR50_START | fmstart | Kontynuacja wyszukiwania |

**Odpowiedź:**

```c
#define GG_PUBDIR50_REPLY        0x000e
#define GG_PUBDIR50_SEARCH_REPLY 0x05
```

---

### 1.11. Lista kontaktów

Od wersji 6.0 lista kontaktów jest częścią sesji.

```c
#define GG_USERLIST_REQUEST 0x0016

struct gg_userlist_request {
    char type;
    char request[];
};

#define GG_USERLIST_PUT      0x00  /* początek eksportu */
#define GG_USERLIST_PUT_MORE 0x01  /* kontynuacja */
#define GG_USERLIST_GET      0x02  /* import */
```

Format listy (CSV):
```
imię;nazwisko;pseudonim;wyświetlane;telefon;grupa;uin;email;dostępny;ścieżka;wiadomość;ścieżka;ukrywanie;telefon_domowy
```

Lista dzielona na pakiety po **2048 bajtów**.

**Odpowiedź:**

```c
#define GG_USERLIST_REPLY          0x0010

#define GG_USERLIST_PUT_REPLY      0x00
#define GG_USERLIST_PUT_MORE_REPLY 0x02
#define GG_USERLIST_GET_MORE_REPLY 0x04
#define GG_USERLIST_GET_REPLY      0x06
```

---

### 1.12. Indeks pakietów

**Pakiety wysyłane (C → S):**

| Wartość | Etykieta | Znaczenie |
|---------|----------|-----------|
| 0x0002 | GG_NEW_STATUS | Zmiana stanu |
| 0x0007 | GG_PONG | Pong |
| 0x0008 | GG_PING | Ping |
| 0x000b | GG_SEND_MSG | Wysłanie wiadomości |
| 0x000c | GG_LOGIN | Logowanie (przed 6.0) |
| 0x000d | GG_ADD_NOTIFY | Dodanie kontaktu |
| 0x000e | GG_REMOVE_NOTIFY | Usunięcie kontaktu |
| 0x000f | GG_NOTIFY_FIRST | Lista kontaktów (początek) |
| 0x0010 | GG_NOTIFY_LAST | Lista kontaktów (koniec) |
| 0x0012 | GG_LIST_EMPTY | Pusta lista |
| 0x0013 | GG_LOGIN_EXT | Logowanie (przed 6.0) |
| 0x0014 | GG_PUBDIR50_REQUEST | Katalog publiczny |
| 0x0015 | GG_LOGIN60 | Logowanie |
| 0x0016 | GG_USERLIST_REQUEST | Lista na serwerze |

**Pakiety odbierane (S → C):**

| Wartość | Etykieta | Znaczenie |
|---------|----------|-----------|
| 0x0001 | GG_WELCOME | Seed hasła |
| 0x0002 | GG_STATUS | Zmiana stanu (przed 6.0) |
| 0x0003 | GG_LOGIN_OK | Logowanie OK |
| 0x0005 | GG_SEND_MSG_ACK | Potwierdzenie |
| 0x0007 | GG_PONG | Pong |
| 0x0008 | GG_PING | Ping |
| 0x0009 | GG_LOGIN_FAILED | Logowanie nieudane |
| 0x000a | GG_RECV_MSG | Wiadomość |
| 0x000b | GG_DISCONNECTING | Rozłączenie |
| 0x000c | GG_NOTIFY_REPLY | Stan kontaktów (przed 6.0) |
| 0x000e | GG_PUBDIR50_REPLY | Odpowiedź katalogu |
| 0x000f | GG_STATUS60 | Zmiana stanu |
| 0x0010 | GG_USERLIST_REPLY | Lista kontaktów |
| 0x0011 | GG_NOTIFY_REPLY60 | Stan kontaktów |

---

## 2. Usługi HTTP

### 2.1. Format danych

```http
POST ŚCIEŻKA HTTP/1.0
Host: HOST
Content-Type: application/x-www-form-urlencoded
User-Agent: AGENT
Content-Length: DŁUGOŚĆ
Pragma: no-cache

DANE
```

Dane: `pole1=wartość1&pole2=wartość2&...` (URL-encoded, CP1250)

---

### 2.2. Tokeny

Każda operacja wymaga autoryzacji tokenem.

**Pobranie tokenu:**
```
HOST: register.gadu-gadu.pl
ŚCIEŻKA: /appsvc/regtoken.asp
```

**Odpowiedź:**
```
SZEROKOŚĆ WYSOKOŚĆ DŁUGOŚĆ
IDENTYFIKATOR
ŚCIEŻKA_OBRAZKA
```

Obrazek tokenu: `http://register.gadu-gadu.pl/appsvc/tokenpic.asp?tokenid=IDENTYFIKATOR`

---

### 2.3. Rejestracja konta

```
HOST: register.gadu-gadu.pl
ŚCIEŻKA: /appsvc/fmregister3.asp
```

| Pole | Znaczenie |
|------|-----------|
| pwd | Hasło |
| email | E-mail |
| tokenid | ID tokenu |
| tokenval | Wartość tokenu |
| code | Hash (email + pwd) |

Sukces: `reg_success:UIN`

---

### 2.4. Usunięcie konta

| Pole | Znaczenie |
|------|-----------|
| fmnumber | Numer |
| fmpwd | Hasło |
| delete | "1" |
| pwd | Losowa liczba |
| email | deletedaccount@gadu-gadu.pl |
| tokenid | ID tokenu |
| tokenval | Wartość tokenu |
| code | Hash |

---

### 2.5. Zmiana hasła

| Pole | Znaczenie |
|------|-----------|
| fmnumber | Numer |
| fmpwd | Stare hasło |
| pwd | Nowe hasło |
| email | Nowy e-mail |
| tokenid | ID tokenu |
| tokenval | Wartość tokenu |
| code | Hash |

---

### 2.6. Przypomnienie hasła

```
HOST: retr.gadu-gadu.pl
ŚCIEŻKA: /appsvc/fmsendpwd3.asp
```

| Pole | Znaczenie |
|------|-----------|
| userid | Numer |
| tokenid | ID tokenu |
| tokenval | Wartość tokenu |
| code | Hash |

Sukces: `pwdsend_success`

---

## 3. Połączenia bezpośrednie

### 3.1. Nawiązanie połączenia

Połączenia bezpośrednie pozwalają przesyłać pliki lub prowadzić rozmowy głosowe bez pośrednictwa serwera. Wymagane jest, aby co najmniej jedna strona miała publiczny adres IP.

**Uwaga:** W połączeniach bezpośrednich pakiety nie mają nagłówka `gg_header`.

```c
struct gg_dcc_welcome {
    int uin;       /* numer strony wywołującej */
    int peer_uin;  /* numer strony wywoływanej */
};

struct gg_dcc_welcome_ack {
    int ack;       /* 0x47414455 = "UDAG" */
};

#define GG_DCC_DIRECTION_IN  0x0002
#define GG_DCC_DIRECTION_OUT 0x0003

struct gg_dcc_direction {
    int type;
};
```

Żądanie połączenia przez CTCP: wiadomość `GG_CLASS_CTCP` z bajtem `0x02`.

---

### 3.2. Przesyłanie plików

```c
#define GG_DCC_REQUEST_SEND 0x0001
#define GG_DCC_FILE_INFO    0x0003

struct gg_dcc_file_info {
    int type;
    int unknown1;  /* 0 */
    int unknown2;  /* 0 */
    struct gg_file_info {
        int dwFileAttributes;
        long long ftCreationTime;
        long long ftLastAccessTime;
        long long ftLastWriteTime;
        int nFileSizeHigh;
        int nFileSizeLow;
        int dwReserved0;
        int dwReserved1;
        char cFileName[262];
        char cAlternateFileName[14];
    } info;
};

#define GG_DCC_SEND_ACK 0x0006

struct gg_dcc_send_ack {
    int type;
    int offset;    /* wznawianie transferu */
    int unknown1;
};

#define GG_DCC_SEND_DATA      0x0003
#define GG_DCC_SEND_DATA_LAST 0x0002

struct gg_dcc_send_data {
    int type;
    int length;
    char data[];
};
```

Domyślny rozmiar pakietu: **4096 bajtów**.

---

### 3.3. Rozmowy głosowe

```c
#define GG_DCC_REQUEST_VOICE 0x0002

#define GG_DCC_VOICE_ACK       0x01
#define GG_DCC_VOICE_DATA      0x03
#define GG_DCC_VOICE_TERMINATE 0x04

struct gg_dcc_voice_data {
    char type;
    int length;
    char data[];
};
```

Kodek: **Microsoft GSM** (biblioteka libgsm z opcją WAV49)

| Wersja | Ramki GSM | Rozmiar |
|--------|-----------|---------|
| < 5.0.5 | 6 | 195 bajtów |
| >= 5.0.5 | 10 + bajt zerowy | 326 bajtów |

---

## 4. Autorzy

- **Wojtek Kaniewski** (wojtekka@irc.pl) - pierwsza wersja opisu, utrzymanie
- **Robert J. Woźny** (speedy@atman.pl) - GG 4.6, poprawki
- **Tomasz Jarzynka** (tomee@cpi.pl) - badanie timeoutów
- **Adam Ludwikowski** (adam.ludwikowski@wp.pl) - wiele poprawek
- **Marek Kozina** (klith@hybrid.art.pl) - czas otrzymania wiadomości
- **Rafał Florek** (raf@regionet.regionet.pl) - połączenia konferencyjne
- **Igor Popik** (igipop@wsfiz.edu.pl) - klasy wiadomości
- **Rafał Cyran** (ajron@wp.pl) - remote_port, CTCP, GG_LOGIN_EXT
- **Piotr Mach** (pm@gadu-gadu.com) - usługi HTTP, GG_LOGIN_EXT
- **Adam Czyciak** (acc@interia.pl) - GG_CLASS_ACK
- **Kamil Dębski** (kdebski@kki.net.pl) - czas w stanach opisowych
- **Paweł Piwowar** (alfapawel@go2.pl) - format czasu
- **Tomasz Chiliński** (chilek@chilan.com) - nowości w 5.0.2
- **Radosław Nowak** (rano@ranosoft.com) - wersja 5.0.3
- **Walerian Sokołowski** - protokół bezpośrednich połączeń
- **Nikodem** (n-d@tlen.pl) - flagi rodzaju użytkownika

---

*Źródło: commit 8395ea9 z 21 grudnia 2004 (libgadu)*
