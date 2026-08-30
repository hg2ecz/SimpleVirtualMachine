// Compact tour of the high-value SVM-C+ language subset.
u8 data[8];
u16 totals[2];

u16 main() {
    u16 i = 0;
    u16 sum = 0;

    puts("SVM-C+ language tour");

    for (i = 0; i < sizeof(data); i++) {
        data[i] = i + 1;
        data[i] += 1;

        if (data[i] == 5) {
            continue;
        }

        sum += data[i];
        if (sum > 30 && i > 2) {
            break;
        }
    }

    do {
        sum--;
    } while (sum > 24 && sum != 0);

    totals[0] = sum;
    totals[1] = sizeof(data) + sizeof(totals);

    if (totals[0] != 0 || totals[1] == 0) {
        puts("language tour complete");
    }

    return totals[0];
}
