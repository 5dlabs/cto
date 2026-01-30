// Code smells for Forge (Unreal/C++) to find:
// - Raw pointers without null checks
// - Tick function doing expensive operations
// - Memory leaks
// - Magic numbers
// - Not using UPROPERTY

#include "PlayerCharacter.h"
#include "Engine/World.h"
#include "Kismet/GameplayStatics.h"

APlayerCharacter::APlayerCharacter()
{
    PrimaryActorTick.bCanEverTick = true;
    
    // Magic number
    Health = 100.0f;
    MoveSpeed = 600.0f;
}

void APlayerCharacter::Tick(float DeltaTime)
{
    Super::Tick(DeltaTime);
    
    // Expensive operation every tick - should be event-driven
    TArray<AActor*> FoundActors;
    UGameplayStatics::GetAllActorsOfClass(GetWorld(), AEnemy::StaticClass(), FoundActors);
    
    for (AActor* Actor : FoundActors)
    {
        // No null check
        AEnemy* Enemy = Cast<AEnemy>(Actor);
        float Distance = FVector::Distance(GetActorLocation(), Enemy->GetActorLocation());
        
        // Magic number
        if (Distance < 500.0f)
        {
            // Memory leak - allocating without cleanup
            FString* DebugMsg = new FString(TEXT("Enemy nearby!"));
            UE_LOG(LogTemp, Warning, TEXT("%s"), **DebugMsg);
        }
    }
}

void APlayerCharacter::FireWeapon()
{
    // Raw pointer without UPROPERTY - won't be GC'd properly
    AProjectile* Bullet = GetWorld()->SpawnActor<AProjectile>(ProjectileClass, GetActorLocation(), GetActorRotation());
    
    // No null check on spawn result
    Bullet->SetVelocity(GetActorForwardVector() * 3000.0f);
}

// Unused function
void APlayerCharacter::UnusedHelper()
{
    int x = 5;
    int y = 10;
}
